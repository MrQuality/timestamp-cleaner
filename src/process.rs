//! Subprocesses never use a shell. Both pipes are drained concurrently with bounded memory.
use crate::{Error, Result};
use std::{
    collections::VecDeque,
    io::Read,
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::SyncSender,
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Default)]
pub struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn check(&self) -> Result<()> {
        if self.0.load(Ordering::Acquire) {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
}

struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn drain(
    mut pipe: impl Read,
    limit: usize,
    progress: Option<SyncSender<u64>>,
) -> std::io::Result<(Vec<u8>, bool)> {
    let mut bytes = VecDeque::with_capacity(limit);
    let mut overflow = false;
    let mut chunk = [0; 8192];
    let mut line = Vec::new();
    loop {
        let count = pipe.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        for byte in &chunk[..count] {
            if bytes.len() == limit {
                bytes.pop_front();
                overflow = true;
            }
            bytes.push_back(*byte);
            if let Some(sender) = &progress {
                if *byte == b'\n' {
                    if let Some(value) = std::str::from_utf8(&line)
                        .ok()
                        .and_then(|s| s.strip_prefix("out_time_us="))
                        .and_then(|s| s.trim().parse().ok())
                    {
                        let _ = sender.try_send(value);
                    }
                    line.clear();
                } else if line.len() < 4096 {
                    line.push(*byte);
                }
            }
        }
    }
    Ok((bytes.into_iter().collect(), overflow))
}

pub fn run(
    command: &mut Command,
    cancel: &Cancellation,
    timeout: Duration,
    stdout_limit: usize,
    progress: Option<SyncSender<u64>>,
) -> Result<Vec<u8>> {
    cancel.check()?;
    if stdout_limit == 0 || stdout_limit > 32 * 1024 * 1024 {
        return Err(Error::Invalid(
            "Subprocess capture limit must be between 1 byte and 32 MiB".into(),
        ));
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let program = command.get_program().to_string_lossy().into_owned();
    let progress_mode = progress.is_some();
    let mut child = ChildGuard(command.spawn().map_err(|e| Error::Tool {
        program: program.clone(),
        details: e.to_string(),
    })?);
    let stdout = child
        .0
        .stdout
        .take()
        .ok_or_else(|| Error::Invalid("Missing stdout pipe".into()))?;
    let stderr = child
        .0
        .stderr
        .take()
        .ok_or_else(|| Error::Invalid("Missing stderr pipe".into()))?;
    let out_thread = thread::spawn(move || drain(stdout, stdout_limit, progress));
    let err_thread = thread::spawn(move || drain(stderr, 64 * 1024, None));
    let started = Instant::now();
    let status = loop {
        if let Err(error) = cancel.check() {
            break Err(error);
        }
        if started.elapsed() > timeout {
            break Err(Error::Timeout);
        }
        match child.0.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) => thread::sleep(Duration::from_millis(25)),
            Err(error) => break Err(Error::Io(error)),
        }
    };
    if status.is_err() {
        let _ = child.0.kill();
        let _ = child.0.wait();
    }
    let out = out_thread
        .join()
        .map_err(|_| Error::Invalid("stdout reader panicked".into()))??;
    let err = err_thread
        .join()
        .map_err(|_| Error::Invalid("stderr reader panicked".into()))??;
    let status = status?;
    cancel.check()?;
    if !status.success() {
        return Err(Error::Tool {
            program,
            details: String::from_utf8_lossy(&err.0).into_owned(),
        });
    }
    if out.1 && !progress_mode {
        return Err(Error::Invalid(
            "Media tool exceeded the output size limit".into(),
        ));
    }
    Ok(out.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_capture_keeps_diagnostics_tail() {
        let (bytes, overflow) = drain(&b"0123456789"[..], 4, None).unwrap();
        assert_eq!(bytes, b"6789");
        assert!(overflow);
    }
    #[test]
    fn cancellation_is_shared_and_prevents_spawn() {
        let token = Cancellation::default();
        token.clone().cancel();
        assert!(matches!(
            run(
                &mut Command::new("does-not-exist"),
                &token,
                Duration::from_secs(1),
                100,
                None
            ),
            Err(Error::Cancelled)
        ));
    }
    #[test]
    fn progress_parser_handles_negative_and_invalid_values() {
        let (tx, rx) = std::sync::mpsc::sync_channel(4);
        drain(
            &b"out_time_us=-10\nout_time_us=wrong\nout_time_us=4000\n"[..],
            100,
            Some(tx),
        )
        .unwrap();
        assert_eq!(rx.try_iter().collect::<Vec<_>>(), vec![4000]);
    }
}
