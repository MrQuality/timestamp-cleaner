"""Repeatable, local, public-fixture measurements; never run an installer.

Windows peak working set is sampled from the OS process counters. Very short-lived
processes can exit between samples, so report this as a sampled lower bound.
"""
import argparse
import ctypes
import functools
import http.server
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile
import threading
import time


class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass


def windows_sampler(process):
    if os.name != "nt":
        return lambda: None, lambda: None
    from ctypes import wintypes

    class Counters(ctypes.Structure):
        _fields_ = [("cb", wintypes.DWORD), ("page_faults", wintypes.DWORD)] + [
            (name, ctypes.c_size_t) for name in (
                "peak_working_set", "working_set", "quota_peak_paged", "quota_paged",
                "quota_peak_nonpaged", "quota_nonpaged", "pagefile", "peak_pagefile",
            )
        ]

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.CloseHandle.restype = wintypes.BOOL
    psapi = ctypes.WinDLL("psapi", use_last_error=True)
    psapi.GetProcessMemoryInfo.argtypes = [wintypes.HANDLE, ctypes.POINTER(Counters), wintypes.DWORD]
    psapi.GetProcessMemoryInfo.restype = wintypes.BOOL
    handle = kernel.OpenProcess(0x0400 | 0x0010, False, process.pid)
    if not handle:
        raise ctypes.WinError(ctypes.get_last_error())

    def sample():
        counters = Counters()
        counters.cb = ctypes.sizeof(counters)
        if psapi.GetProcessMemoryInfo(handle, ctypes.byref(counters), counters.cb):
            return counters.peak_working_set
        return None

    return sample, lambda: kernel.CloseHandle(handle)


def measure(binary, fixture, state, base):
    args = [str(binary), str(fixture / "bootstrap-root.json"),
            base + "metadata/", base + "targets/", str(state)]
    started = time.perf_counter()
    process = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    peak, samples = None, 0
    close = lambda: None
    try:
        sample, close = windows_sampler(process)
        while process.poll() is None:
            value = sample()
            if value is not None:
                peak = max(peak or 0, value)
                samples += 1
            if time.perf_counter() - started > 20:
                raise TimeoutError("Probe exceeded benchmark deadline")
            time.sleep(0.001)
        output, errors = process.communicate(timeout=2)
        elapsed = (time.perf_counter() - started) * 1000
        if process.returncode:
            raise RuntimeError(errors.decode(errors="replace"))
        assert json.loads(output)["version"] == "0.1.0-beta.2"
        return {"elapsed_ms": round(elapsed, 3),
                "sampled_peak_working_set_bytes": peak, "memory_samples": samples}
    finally:
        if process.poll() is None:
            process.kill()
        process.communicate(timeout=2)
        close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("public_fixture", type=Path)
    parser.add_argument("--rounds", type=int, default=5)
    args = parser.parse_args()
    if not 1 <= args.rounds <= 20:
        parser.error("rounds must be between 1 and 20")
    binary, fixture = args.binary.resolve(strict=True), args.public_fixture.resolve(strict=True)
    assert (fixture / "bootstrap-root.json").is_file()
    handler = functools.partial(QuietHandler, directory=str(fixture))
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    worker = threading.Thread(target=server.serve_forever, kwargs={"poll_interval": 0.05}, daemon=True)
    worker.start()
    results = []
    try:
        with tempfile.TemporaryDirectory(prefix="tuf-probe-measure-") as work:
            for transport, base in [
                ("file", fixture.as_uri() + "/"),
                ("http-loopback", f"http://127.0.0.1:{server.server_port}/"),
            ]:
                for trial in range(args.rounds):
                    state = Path(work) / f"{transport}-{trial}"
                    for condition in ["fresh-state", "persisted-state"]:
                        result = measure(binary, fixture, state, base)
                        results.append(dict(transport=transport, condition=condition, trial=trial, **result))
    finally:
        server.shutdown()
        server.server_close()
        worker.join(timeout=2)
    print(json.dumps({"platform": platform.platform(), "python": platform.python_version(),
                      "binary_bytes": binary.stat().st_size, "results": results,
                      "limits": ["Separate process per trial; persisted-state does not reuse an HTTP client",
                                 "Loopback HTTP excludes TLS, real CDN/proxy and internet latency",
                                 "Sampled Windows OS peak is a lower bound; non-Windows memory is unmeasured",
                                 "Includes process creation and sampling overhead; not application incremental cost"]}, indent=2))


if __name__ == "__main__":
    main()
