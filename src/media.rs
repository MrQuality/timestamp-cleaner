use crate::{
    Error, Result,
    model::Region,
    process::{self, Cancellation},
};
use serde::Deserialize;
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

const FORMATS: &str = "mov,matroska,webm,avi,mpegts,mpeg,ogg";

#[derive(Clone, Debug)]
pub struct Tools {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
}

impl Tools {
    pub fn discover() -> Result<Self> {
        Ok(Self {
            ffmpeg: executable("ffmpeg")?,
            ffprobe: executable("ffprobe")?,
        })
    }
}

fn executable(name: &str) -> Result<PathBuf> {
    let key = format!("TIMESTAMP_CLEANER_{}", name.to_uppercase());
    if let Some(value) = std::env::var_os(&key) {
        let path = PathBuf::from(value);
        if path.is_absolute() && path.is_file() {
            return Ok(path.canonicalize()?);
        }
        return Err(Error::Invalid(format!(
            "{key} must name an absolute executable file"
        )));
    }
    let filename = format!("{name}{}", std::env::consts::EXE_SUFFIX);
    let cwd = std::env::current_dir()?.canonicalize()?;
    if let Some(paths) = std::env::var_os("PATH") {
        for folder in std::env::split_paths(&paths) {
            // Do not execute tools supplied by the current media/project directory.
            if !folder.is_absolute() || folder.canonicalize().ok().as_ref() == Some(&cwd) {
                continue;
            }
            let candidate = folder.join(&filename);
            if candidate.is_file() {
                return Ok(candidate.canonicalize()?);
            }
        }
    }
    Err(Error::Invalid(format!(
        "Install a trusted FFmpeg build containing {filename}, add it to PATH, or set {key}"
    )))
}

#[derive(Debug, Deserialize)]
struct Probe {
    streams: Vec<Stream>,
    format: Format,
}
#[derive(Debug, Deserialize)]
struct Format {
    duration: Option<String>,
}
#[derive(Debug, Deserialize)]
struct Stream {
    index: u32,
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    pix_fmt: Option<String>,
    color_transfer: Option<String>,
    sample_aspect_ratio: Option<String>,
    #[serde(default)]
    side_data_list: Vec<SideData>,
    #[serde(default)]
    disposition: Disposition,
}
#[derive(Debug, Default, Deserialize)]
struct Disposition {
    #[serde(default)]
    attached_pic: u32,
}
#[derive(Debug, Deserialize)]
struct SideData {
    rotation: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct Media {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub duration: f64,
    pub video_index: u32,
    pub audio_codecs: Vec<String>,
}

/// Use the same restrictions for probing, previews and exports. No playlists or network protocols.
pub fn input(command: &mut Command, path: &Path) {
    command.args(["-threads", "4"]);
    command
        .args([
            "-protocol_whitelist",
            "file,pipe",
            "-format_whitelist",
            FORMATS,
            "-noautorotate",
            "-i",
        ])
        .arg(path);
}

pub fn probe(tools: &Tools, path: &Path, cancel: &Cancellation) -> Result<Media> {
    let path = path.canonicalize()?;
    if !path.is_file() {
        return Err(Error::Invalid("Input must be a local regular file".into()));
    }
    let mut command = Command::new(&tools.ffprobe);
    command.args(["-v", "error", "-protocol_whitelist", "file,pipe", "-format_whitelist", FORMATS,
        "-show_entries", "stream=index,codec_type,codec_name,width,height,pix_fmt,color_transfer,sample_aspect_ratio:stream_side_data=rotation:stream_disposition=attached_pic:format=duration",
        "-of", "json"]).arg(&path);
    let bytes = process::run(
        &mut command,
        cancel,
        Duration::from_secs(30),
        1024 * 1024,
        None,
    )?;
    parse_probe(path, &bytes)
}

fn parse_probe(path: PathBuf, bytes: &[u8]) -> Result<Media> {
    let parsed: Probe = serde_json::from_slice(bytes)?;
    let video = parsed
        .streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("video") && s.disposition.attached_pic == 0)
        .ok_or_else(|| Error::Invalid("No playable video stream found".into()))?;
    let width = video.width.unwrap_or(0);
    let height = video.height.unwrap_or(0);
    let duration: f64 = parsed
        .format
        .duration
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    if width < 4
        || height < 4
        || width > 8192
        || height > 8192
        || u64::from(width) * u64::from(height) > 33_554_432
        || !width.is_multiple_of(2)
        || !height.is_multiple_of(2)
    {
        return Err(Error::Invalid("This release supports even video dimensions up to 8192 pixels per side and 32 megapixels".into()));
    }
    if !duration.is_finite() || duration <= 0.0 {
        return Err(Error::Invalid(
            "Video must have a finite, positive duration".into(),
        ));
    }
    if video
        .side_data_list
        .iter()
        .any(|s| s.rotation.is_some_and(|r| r.rem_euclid(360) != 0))
    {
        return Err(Error::Invalid(
            "Rotated video is not supported yet; normalize its orientation before opening".into(),
        ));
    }
    if video
        .sample_aspect_ratio
        .as_deref()
        .is_some_and(|s| !["1:1", "0:1", "N/A"].contains(&s))
    {
        return Err(Error::Invalid(
            "Non-square video pixels are not supported yet".into(),
        ));
    }
    if !matches!(
        video.pix_fmt.as_deref(),
        Some(
            "yuv420p"
                | "yuv422p"
                | "yuv444p"
                | "yuvj420p"
                | "yuvj422p"
                | "yuvj444p"
                | "rgb24"
                | "bgr24"
                | "nv12"
                | "gray"
        )
    ) {
        return Err(Error::Invalid("This release supports SDR, 8-bit opaque video; high-bit-depth and alpha video require a dedicated color pipeline".into()));
    }
    if matches!(
        video.color_transfer.as_deref(),
        Some("smpte2084" | "arib-std-b67")
    ) {
        return Err(Error::Invalid(
            "HDR transfer functions require a dedicated color pipeline".into(),
        ));
    }
    let audio_codecs = parsed
        .streams
        .iter()
        .filter(|s| s.codec_type.as_deref() == Some("audio"))
        .map(|s| s.codec_name.clone().unwrap_or_default())
        .collect();
    Ok(Media {
        path,
        width,
        height,
        duration,
        video_index: video.index,
        audio_codecs,
    })
}

pub fn preview(
    tools: &Tools,
    media: &Media,
    time: f64,
    region: Option<Region>,
    cancel: &Cancellation,
) -> Result<image::RgbaImage> {
    if !time.is_finite() || time < 0.0 || time >= media.duration {
        return Err(Error::Invalid("Preview time is outside the video".into()));
    }
    let filter = match region {
        Some(r) => format!(
            "{},scale=1280:720:force_original_aspect_ratio=decrease",
            r.validate(media.width, media.height)?.filter()
        ),
        None => "scale=1280:720:force_original_aspect_ratio=decrease".into(),
    };
    let mut command = Command::new(&tools.ffmpeg);
    command.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-xerror"]);
    // Fast seek before decoding; FFmpeg accurate seeking discards frames before the requested time.
    command.args(["-ss", &format!("{time:.6}")]);
    input(&mut command, &media.path);
    command.args([
        "-map",
        &format!("0:{}", media.video_index),
        "-an",
        "-sn",
        "-dn",
        "-vf",
        &filter,
        "-frames:v",
        "1",
        "-threads",
        "2",
        "-f",
        "image2pipe",
        "-c:v",
        "png",
        "pipe:1",
    ]);
    let bytes = process::run(
        &mut command,
        cancel,
        Duration::from_secs(30),
        8 * 1024 * 1024,
        None,
    )?;
    Ok(image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)?.to_rgba8())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(width: u32, height: u32, duration: &str, extra: &str) -> Vec<u8> {
        format!(r#"{{"streams":[{{"index":0,"codec_type":"video","width":{width},"height":{height},"pix_fmt":"yuv420p"{extra}}}],"format":{{"duration":"{duration}"}}}}"#).into_bytes()
    }
    #[test]
    fn rejects_invalid_media_before_decode() {
        for bytes in [
            fixture(0, 720, "1", ""),
            fixture(1279, 720, "1", ""),
            fixture(8192, 8192, "1", ""),
            fixture(1280, 720, "NaN", ""),
            fixture(1280, 720, "0", ""),
            fixture(1280, 720, "1", r#", "side_data_list":[{"rotation":90}]"#),
            fixture(1280, 720, "1", r#", "sample_aspect_ratio":"4:3""#),
        ] {
            assert!(parse_probe("test.mp4".into(), &bytes).is_err());
        }
    }
    #[test]
    fn reads_video_dimensions_without_fps_fallback() {
        let media = parse_probe("test.mp4".into(), &fixture(1280, 720, "1.25", "")).unwrap();
        assert_eq!((media.width, media.height), (1280, 720));
        assert_eq!(media.duration, 1.25);
    }
}
