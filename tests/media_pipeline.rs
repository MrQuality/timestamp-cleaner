#![cfg(feature = "media-tests")]
//! Explicit opt-in: these tests REQUIRE a trusted FFmpeg installation, never silently skip.
use std::{path::Path, process::Command, sync::mpsc, thread, time::Duration};
use timestamp_cleaner::{
    Error,
    export::{ExportRequest, export},
    media::{Media, Tools, preview, probe},
    model::{Quality, Region},
    process::{self, Cancellation},
};

fn command_output(command: &mut Command) -> Vec<u8> {
    let result = command.output().expect("start fixture tool");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    result.stdout
}

fn fixture(tools: &Tools, path: &Path, variable_rate: bool) -> Media {
    let mut command = Command::new(&tools.ffmpeg);
    command.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostdin",
        "-f",
        "lavfi",
        "-i",
        "testsrc2=size=160x96:rate=25:duration=1.2",
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=440:duration=1.2",
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=880:duration=1.2",
        "-map",
        "0:v",
        "-map",
        "1:a",
        "-map",
        "2:a",
    ]);
    if variable_rate {
        command.args(["-vf", "select='not(eq(mod(n,3),2))'", "-fps_mode", "vfr"]);
    }
    command
        .args([
            "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-threads", "2",
        ])
        .arg(path);
    command_output(&mut command);
    probe(tools, path, &Cancellation::default()).unwrap()
}

fn request(source: Media, destination: &Path) -> ExportRequest {
    ExportRequest {
        source,
        destination: destination.into(),
        region: Region {
            x: 20,
            y: 20,
            width: 40,
            height: 20,
        },
        quality: Quality::High,
    }
}

fn timestamps(tools: &Tools, path: &Path) -> Vec<f64> {
    let bytes = command_output(
        Command::new(&tools.ffprobe)
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "frame=best_effort_timestamp_time",
                "-of",
                "json",
            ])
            .arg(path),
    );
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["frames"]
        .as_array()
        .unwrap()
        .iter()
        .map(|frame| {
            frame["best_effort_timestamp_time"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap()
        })
        .collect()
}

fn audio_hash(tools: &Tools, path: &Path) -> Vec<u8> {
    command_output(
        Command::new(&tools.ffmpeg)
            .args(["-hide_banner", "-loglevel", "error", "-i"])
            .arg(path)
            .args([
                "-map",
                "0:a",
                "-c",
                "copy",
                "-f",
                "streamhash",
                "-hash",
                "sha256",
                "pipe:1",
            ]),
    )
}

fn audio_timestamps(tools: &Tools, path: &Path) -> Vec<f64> {
    let bytes = command_output(
        Command::new(&tools.ffprobe)
            .args([
                "-v",
                "error",
                "-select_streams",
                "a",
                "-show_entries",
                "packet=pts_time",
                "-of",
                "json",
            ])
            .arg(path),
    );
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["packets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|packet| packet["pts_time"].as_str().unwrap().parse().unwrap())
        .collect()
}

#[test]
fn export_preserves_audio_payloads_frame_count_and_variable_timing() {
    let tools = Tools::discover().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("משפחה trip & $(literal).mp4");
    let source = fixture(&tools, &path, true);
    let original_bytes = std::fs::read(&path).unwrap();
    let destination = temp.path().join("selected repair.mkv");
    let result = export(
        &tools,
        &request(source.clone(), &destination),
        &Cancellation::default(),
        None,
    )
    .unwrap();
    let output = probe(&tools, &result, &Cancellation::default()).unwrap();
    assert_eq!((output.width, output.height), (160, 96));
    assert_eq!(output.audio_codecs, vec!["aac", "aac"]);
    assert_eq!(audio_hash(&tools, &path), audio_hash(&tools, &result));
    let audio_before = audio_timestamps(&tools, &path);
    let audio_after = audio_timestamps(&tools, &result);
    assert_eq!(audio_before.len(), audio_after.len());
    for (a, b) in audio_before.iter().zip(&audio_after) {
        assert!((a - b).abs() < 0.002, "audio timestamp changed: {a} -> {b}");
    }
    let before = timestamps(&tools, &path);
    let after = timestamps(&tools, &result);
    assert_eq!(before.len(), after.len());
    assert!(
        before.windows(2).any(|pair| pair[1] - pair[0] > 0.06),
        "fixture must really be VFR"
    );
    for (a, b) in before.iter().zip(&after) {
        assert!((a - b).abs() < 0.002, "timestamp changed: {a} -> {b}");
    }
    assert_eq!(std::fs::read(path).unwrap(), original_bytes);
    assert_eq!(
        std::fs::read_dir(temp.path()).unwrap().count(),
        2,
        "temporary output leaked"
    );
    // Frame preview and export use the identical source-coordinate repair filter.
    let original = preview(&tools, &source, 0.0, None, &Cancellation::default()).unwrap();
    let repaired = preview(
        &tools,
        &source,
        0.0,
        Some(request(source.clone(), &destination).region),
        &Cancellation::default(),
    )
    .unwrap();
    assert_eq!(original.dimensions(), repaired.dimensions());
    assert_ne!(
        original.as_raw(),
        repaired.as_raw(),
        "repair must change image content"
    );
}

#[test]
fn existing_destination_is_never_overwritten() {
    let tools = Tools::discover().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let source = fixture(&tools, &temp.path().join("source.mp4"), false);
    let destination = temp.path().join("existing.mkv");
    std::fs::write(&destination, b"keep this result").unwrap();
    assert!(
        export(
            &tools,
            &request(source, &destination),
            &Cancellation::default(),
            None
        )
        .is_err()
    );
    assert_eq!(std::fs::read(destination).unwrap(), b"keep this result");
}

#[test]
fn failing_encoder_and_cancelled_export_publish_nothing() {
    let tools = Tools::discover().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let source = fixture(&tools, &temp.path().join("source.mp4"), false);
    let destination = temp.path().join("failed.mkv");
    let request = request(source, &destination);
    let broken = Tools {
        ffmpeg: temp.path().join("missing-encoder.exe"),
        ffprobe: tools.ffprobe.clone(),
    };
    assert!(export(&broken, &request, &Cancellation::default(), None).is_err());
    assert!(!destination.exists());
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        export(&tools, &request, &cancel, None),
        Err(Error::Cancelled)
    ));
    assert!(!destination.exists());
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
}

#[test]
fn subprocess_timeout_and_midflight_cancellation_reap_the_encoder() {
    let tools = Tools::discover().unwrap();
    for timed_out in [true, false] {
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let ffmpeg = tools.ffmpeg.clone();
        let (tx, rx) = mpsc::sync_channel(4);
        let handle = thread::spawn(move || {
            process::run(
                Command::new(ffmpeg).args([
                    "-hide_banner",
                    "-loglevel",
                    "error",
                    "-nostdin",
                    "-re",
                    "-f",
                    "lavfi",
                    "-i",
                    "testsrc2=size=160x96:rate=25",
                    "-progress",
                    "pipe:1",
                    "-f",
                    "null",
                    "-",
                ]),
                &worker_cancel,
                if timed_out {
                    Duration::from_millis(250)
                } else {
                    Duration::from_secs(10)
                },
                4096,
                Some(tx),
            )
        });
        if !timed_out {
            rx.recv_timeout(Duration::from_secs(5))
                .expect("encoder began processing");
            cancel.cancel();
        }
        let result = handle.join().unwrap();
        if timed_out {
            assert!(matches!(result, Err(Error::Timeout)));
        } else {
            assert!(matches!(result, Err(Error::Cancelled)));
        }
    }
}

#[test]
fn playlists_and_corrupt_files_are_rejected() {
    let tools = Tools::discover().unwrap();
    let temp = tempfile::tempdir().unwrap();
    for (name, content) in [
        ("corrupt.mp4", "not a video"),
        (
            "remote.m3u8",
            "#EXTM3U\n#EXTINF:1,\nhttps://example.invalid/segment.ts\n",
        ),
    ] {
        let path = temp.path().join(name);
        std::fs::write(&path, content).unwrap();
        assert!(probe(&tools, &path, &Cancellation::default()).is_err());
    }
}

#[test]
fn cancellation_and_destination_race_remove_partial_exports() {
    let tools = Tools::discover().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let short = fixture(&tools, &temp.path().join("short.mp4"), false);
    let long_path = temp.path().join("long.mp4");
    command_output(
        Command::new(&tools.ffmpeg)
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-stream_loop",
                "99",
                "-i",
            ])
            .arg(&short.path)
            .args(["-map", "0", "-c", "copy"])
            .arg(&long_path),
    );
    let source = probe(&tools, &long_path, &Cancellation::default()).unwrap();
    for cancel_job in [true, false] {
        let destination = temp.path().join(if cancel_job {
            "cancelled.mkv"
        } else {
            "collision.mkv"
        });
        let request = request(source.clone(), &destination);
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let worker_tools = tools.clone();
        let (tx, rx) = mpsc::sync_channel(16);
        let handle =
            thread::spawn(move || export(&worker_tools, &request, &worker_cancel, Some(tx)));
        rx.recv_timeout(Duration::from_secs(20))
            .expect("export started");
        if cancel_job {
            cancel.cancel();
        } else {
            std::fs::write(&destination, b"concurrent result").unwrap();
        }
        let result = handle.join().unwrap();
        if cancel_job {
            assert!(matches!(result, Err(Error::Cancelled)));
            assert!(!destination.exists());
        } else {
            assert!(
                matches!(result, Err(Error::Io(ref error)) if error.kind() == std::io::ErrorKind::AlreadyExists),
                "expected publication collision, got {result:?}"
            );
            assert_eq!(std::fs::read(&destination).unwrap(), b"concurrent result");
        }
        assert!(!std::fs::read_dir(temp.path()).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".partial")
        }));
    }
}
