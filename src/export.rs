use crate::{
    Error, Result,
    media::{self, Media, Tools},
    model::{Quality, Region},
    process::{self, Cancellation},
};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc::SyncSender,
    time::Duration,
};

pub struct ExportRequest {
    pub source: Media,
    pub destination: PathBuf,
    pub region: Region,
    pub quality: Quality,
}

/// Matroska supports copying a broad range of source audio codecs without lossy conversion.
fn validate_destination(path: &Path) -> Result<PathBuf> {
    if path
        .extension()
        .is_none_or(|ext| !ext.eq_ignore_ascii_case("mkv"))
    {
        return Err(Error::Invalid(
            "Choose a .mkv output file to preserve source audio codecs".into(),
        ));
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = parent.canonicalize()?;
    let name = path
        .file_name()
        .ok_or_else(|| Error::Invalid("Output needs a file name".into()))?;
    let destination = parent.join(name);
    // symlink_metadata also rejects dangling symlinks. persist_noclobber handles the final race.
    if destination.symlink_metadata().is_ok() {
        return Err(Error::Invalid(
            "Output already exists; choose a new name".into(),
        ));
    }
    Ok(destination)
}

pub fn export(
    tools: &Tools,
    request: &ExportRequest,
    cancel: &Cancellation,
    progress: Option<SyncSender<u64>>,
) -> Result<PathBuf> {
    cancel.check()?;
    let region = request
        .region
        .validate(request.source.width, request.source.height)?;
    let destination = validate_destination(&request.destination)?;
    let source = media::probe(tools, &request.source.path, cancel)?;
    // Re-probe rather than trusting stale UI metadata after external source edits.
    if source.width != request.source.width
        || source.height != request.source.height
        || (source.duration - request.source.duration).abs() > 0.001
        || source.audio_codecs != request.source.audio_codecs
    {
        return Err(Error::Invalid(
            "Source media changed; reopen it before exporting".into(),
        ));
    }
    let temporary = tempfile::Builder::new()
        .prefix(".timestamp-cleaner-")
        .suffix(".partial")
        .tempfile_in(
            destination
                .parent()
                .ok_or_else(|| Error::Invalid("Missing output folder".into()))?,
        )?;
    let mut command = Command::new(&tools.ffmpeg);
    command.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostdin",
        "-xerror",
        "-y",
        "-copyts",
        "-start_at_zero",
        "-filter_threads",
        "2",
    ]);
    media::input(&mut command, &source.path);
    command
        .args([
            "-map",
            &format!("0:{}", source.video_index),
            "-map",
            "0:a?",
            "-map_metadata",
            "0",
            "-map_chapters",
            "0",
            "-vf",
            &region.filter(),
            "-c:v",
            "libx264",
            "-preset",
            "medium",
            "-crf",
            request.quality.crf(),
            "-pix_fmt",
            "yuv420p",
            "-fps_mode",
            "passthrough",
            "-enc_time_base",
            "demux",
            "-avoid_negative_ts",
            "disabled",
            "-c:a",
            "copy",
            "-threads",
            "4",
            "-progress",
            "pipe:1",
            "-nostats",
            "-f",
            "matroska",
        ])
        .arg(temporary.path());
    let progress = progress.unwrap_or_else(|| std::sync::mpsc::sync_channel(1).0);
    process::run(
        &mut command,
        cancel,
        Duration::from_secs(24 * 60 * 60),
        64 * 1024,
        Some(progress),
    )?;
    cancel.check()?;
    let result = media::probe(tools, temporary.path(), cancel)?;
    validate_output(&source, &result)?;
    temporary.as_file().sync_all()?;
    cancel.check()?;
    // Atomic no-clobber publication on the same filesystem. Never overwrite an existing file.
    temporary
        .persist_noclobber(&destination)
        .map_err(|e| Error::Io(e.error))?;
    Ok(destination)
}

fn validate_output(source: &Media, result: &Media) -> Result<()> {
    if (source.width, source.height) != (result.width, result.height)
        || source.audio_codecs != result.audio_codecs
        || (source.duration - result.duration).abs() > 0.25
    {
        return Err(Error::Invalid(
            "Export validation failed: dimensions, audio streams, or duration changed".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_existing_outputs_and_wrong_containers() {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("keep.mkv");
        std::fs::write(&output, b"original").unwrap();
        assert!(validate_destination(&output).is_err());
        assert_eq!(std::fs::read(&output).unwrap(), b"original");
        assert!(validate_destination(&temp.path().join("out.mp4")).is_err());
        assert!(validate_destination(&temp.path().join("new.mkv")).is_ok());
    }
    #[test]
    fn refuses_truncated_or_silent_export() {
        let source = Media {
            path: "input.mp4".into(),
            width: 1280,
            height: 720,
            duration: 10.0,
            video_index: 0,
            audio_codecs: vec!["aac".into()],
        };
        let mut result = source.clone();
        result.duration = 9.0;
        assert!(validate_output(&source, &result).is_err());
        result.duration = 10.0;
        result.audio_codecs.clear();
        assert!(validate_output(&source, &result).is_err());
        result.audio_codecs = source.audio_codecs.clone();
        assert!(validate_output(&source, &result).is_ok());
    }
}
