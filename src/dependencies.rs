//! Local tool discovery is separate from installation and never makes network requests.
use crate::{
    Error, Result,
    process::{self, Cancellation},
};
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

#[derive(Clone, Debug)]
pub struct Tools {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
}

#[derive(Clone, Debug)]
pub enum Availability {
    Ready(PathBuf),
    Unavailable(String),
}

#[derive(Clone, Debug)]
pub struct DependencyReport {
    pub ffmpeg: Availability,
    pub ffprobe: Availability,
}

impl DependencyReport {
    pub fn detect(cancel: &Cancellation) -> Self {
        let detect = |name| match resolve_system(name).and_then(|path| {
            verify(name, &path, cancel)?;
            Ok(path)
        }) {
            Ok(path) => Availability::Ready(path),
            Err(error) => Availability::Unavailable(error.to_string()),
        };
        Self {
            ffmpeg: detect("ffmpeg"),
            ffprobe: detect("ffprobe"),
        }
    }

    pub fn tools(&self) -> Option<Tools> {
        match (&self.ffmpeg, &self.ffprobe) {
            (Availability::Ready(ffmpeg), Availability::Ready(ffprobe)) => Some(Tools {
                ffmpeg: ffmpeg.clone(),
                ffprobe: ffprobe.clone(),
            }),
            _ => None,
        }
    }

    pub fn problems(&self) -> Vec<String> {
        [("ffmpeg", &self.ffmpeg), ("ffprobe", &self.ffprobe)]
            .into_iter()
            .filter_map(|(name, status)| match status {
                Availability::Unavailable(reason) => Some(format!("{name}: {reason}")),
                Availability::Ready(_) => None,
            })
            .collect()
    }
}

impl Tools {
    pub fn discover() -> Result<Self> {
        let report = DependencyReport::detect(&Cancellation::default());
        report
            .tools()
            .ok_or_else(|| Error::Invalid(report.problems().join("\n")))
    }
}

fn search_directories() -> Vec<PathBuf> {
    let mut folders: Vec<_> = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).collect())
        .unwrap_or_default();
    if cfg!(windows) {
        // WinGet links must be read explicitly: the running process retains its old PATH after installation.
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            folders.push(PathBuf::from(&local).join("Microsoft/WinGet/Links"));
            folders.push(PathBuf::from(local).join("Microsoft/WindowsApps"));
        }
        if let Some(programs) = std::env::var_os("ProgramFiles") {
            folders.push(PathBuf::from(programs).join("WinGet/Links"));
        }
    }
    folders
}

pub(crate) fn resolve_system(name: &str) -> Result<PathBuf> {
    let key = format!("TIMESTAMP_CLEANER_{}", name.to_uppercase());
    let explicit = std::env::var_os(&key).map(PathBuf::from);
    resolve(
        name,
        explicit.as_deref(),
        &search_directories(),
        &std::env::current_dir()?.canonicalize()?,
    )
}

fn resolve(
    name: &str,
    explicit: Option<&Path>,
    folders: &[PathBuf],
    cwd: &Path,
) -> Result<PathBuf> {
    if let Some(path) = explicit {
        if path.is_absolute() && path.is_file() {
            return Ok(path.canonicalize()?);
        }
        return Err(Error::Invalid(format!(
            "TIMESTAMP_CLEANER_{} must name an absolute executable file",
            name.to_uppercase()
        )));
    }
    let filename = format!("{name}{}", std::env::consts::EXE_SUFFIX);
    for folder in folders {
        if !folder.is_absolute() || folder.canonicalize().ok().as_deref() == Some(cwd) {
            continue;
        }
        let candidate = folder.join(&filename);
        if candidate.is_file() {
            return Ok(candidate.canonicalize()?);
        }
    }
    Err(Error::Invalid(format!(
        "{filename} was not found in the system tool locations"
    )))
}

fn verify(name: &str, path: &Path, cancel: &Cancellation) -> Result<()> {
    verify_with(name, path, |command| {
        process::run(command, cancel, Duration::from_secs(10), 64 * 1024, None)
    })
}

fn verify_with(
    name: &str,
    path: &Path,
    mut run: impl FnMut(&mut Command) -> Result<Vec<u8>>,
) -> Result<()> {
    let bytes = run(Command::new(path).arg("-version"))?;
    if !bytes.starts_with(format!("{name} version ").as_bytes()) {
        return Err(Error::Invalid(format!(
            "{} did not identify itself as {name}",
            path.display()
        )));
    }
    if name == "ffmpeg" {
        // Exercise the actual timing options rather than guessing capabilities from
        // release/nightly version strings. One synthetic frame is discarded in memory.
        let mut command = Command::new(path);
        command.args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-f",
            "lavfi",
            "-i",
            "color=size=16x16:rate=1",
            "-frames:v",
            "1",
            "-filter_threads",
            "1",
            "-vf",
            "delogo=x=4:y=4:w=4:h=4",
            "-c:v",
            "libx264",
            "-threads",
            "1",
            "-fps_mode",
            "passthrough",
            "-enc_time_base",
            "demux",
            "-f",
            "null",
            "-",
        ]);
        run(&mut command).map_err(|error| match error {
            Error::Cancelled => Error::Cancelled,
            error => Error::Invalid(format!(
                "FFmpeg cannot run the required repair/timing pipeline. Install a current build with libx264, delogo and lavfi support. Details: {error}"
            )),
        })?;
    } else if name == "ffprobe" {
        let sections = run(Command::new(path).args(["-v", "error", "-sections"]))?;
        if !sections
            .split(u8::is_ascii_whitespace)
            .any(|word| word.rsplit(|b| *b == b'/').next() == Some(b"stream_side_data"))
        {
            return Err(Error::Invalid(
                "FFprobe lacks the stream_side_data schema required for rotation checks. Install a current FFmpeg/FFprobe package.".into(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_valid_version_banner_does_not_enable_an_incompatible_ffmpeg() {
        let mut calls = 0;
        let error = verify_with("ffmpeg", Path::new("ffmpeg"), |_| {
            calls += 1;
            if calls == 1 {
                Ok(b"ffmpeg version 4.4.2".to_vec())
            } else {
                Err(Error::Invalid("Unrecognized option fps_mode".into()))
            }
        })
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("required repair/timing pipeline")
        );
        assert_eq!(calls, 2);
    }

    #[test]
    fn executable_identity_is_required_and_capability_cancellation_is_preserved() {
        let mut calls = 0;
        assert!(
            verify_with("ffmpeg", Path::new("wrong-tool"), |_| {
                calls += 1;
                Ok(b"another tool".to_vec())
            })
            .is_err()
        );
        assert_eq!(calls, 1);
        let mut calls = 0;
        assert!(matches!(
            verify_with("ffmpeg", Path::new("ffmpeg"), |_| {
                calls += 1;
                if calls == 1 {
                    Ok(b"ffmpeg version N-114971".to_vec())
                } else {
                    Err(Error::Cancelled)
                }
            }),
            Err(Error::Cancelled)
        ));
        assert!(
            verify_with("ffmpeg", Path::new("ffmpeg"), |command| {
                if command.get_args().any(|a| a == "-version") {
                    Ok(b"ffmpeg version N-114971".to_vec())
                } else {
                    Ok(Vec::new())
                }
            })
            .is_ok()
        );
    }

    #[test]
    fn ffprobe_requires_the_actual_stream_side_data_section() {
        for (sections, supported) in [
            (".... side_data/stream_side_data", true),
            (".... stream_side_data", true),
            (".... side_data_list/stream_side_data_list", false),
            (".... side_data/frame_side_data", false),
        ] {
            let result = verify_with("ffprobe", Path::new("ffprobe"), |command| {
                if command.get_args().any(|a| a == "-version") {
                    Ok(b"ffprobe version 8.1".to_vec())
                } else {
                    Ok(sections.as_bytes().to_vec())
                }
            });
            assert_eq!(result.is_ok(), supported, "{sections}");
        }
    }

    #[test]
    fn discovers_components_independently() {
        let report = DependencyReport {
            ffmpeg: Availability::Ready("ffmpeg".into()),
            ffprobe: Availability::Unavailable("not found".into()),
        };
        assert!(report.tools().is_none());
        assert_eq!(report.problems(), vec!["ffprobe: not found"]);
        let missing = DependencyReport {
            ffmpeg: Availability::Unavailable("not found".into()),
            ffprobe: Availability::Unavailable("not found".into()),
        };
        assert_eq!(missing.problems().len(), 2);
        let ready = DependencyReport {
            ffmpeg: Availability::Ready("ffmpeg".into()),
            ffprobe: Availability::Ready("ffprobe".into()),
        };
        assert!(ready.tools().is_some());
        assert!(ready.problems().is_empty());
    }
    #[test]
    fn ignores_current_and_relative_search_directories() {
        let temp = tempfile::tempdir().unwrap();
        let cwd = temp.path().canonicalize().unwrap();
        std::fs::write(
            cwd.join(format!("ffmpeg{}", std::env::consts::EXE_SUFFIX)),
            b"untrusted",
        )
        .unwrap();
        assert!(resolve("ffmpeg", None, &[cwd.clone(), ".".into()], &cwd).is_err());
    }
    #[test]
    fn explicit_configuration_has_precedence_and_invalid_configuration_is_not_ignored() {
        let temp = tempfile::tempdir().unwrap();
        let tool = temp
            .path()
            .join(format!("ffprobe{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&tool, b"fixture").unwrap();
        let cwd = temp.path().canonicalize().unwrap();
        assert_eq!(
            resolve("ffprobe", Some(&tool), &[], &cwd).unwrap(),
            tool.canonicalize().unwrap()
        );
        assert!(
            resolve(
                "ffprobe",
                Some(Path::new("relative.exe")),
                std::slice::from_ref(&cwd),
                &cwd
            )
            .is_err()
        );
        assert!(
            resolve(
                "ffprobe",
                Some(&cwd.join("missing.exe")),
                std::slice::from_ref(&cwd),
                &cwd
            )
            .is_err()
        );
    }
    #[test]
    fn discovers_new_package_manager_links_without_a_path_refresh() {
        let temp = tempfile::tempdir().unwrap();
        let links = temp.path().join("Microsoft/WinGet/Links");
        std::fs::create_dir_all(&links).unwrap();
        let tool = links.join(format!("ffprobe{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&tool, b"fixture").unwrap();
        assert_eq!(
            resolve(
                "ffprobe",
                None,
                &[links],
                &temp.path().canonicalize().unwrap()
            )
            .unwrap(),
            tool.canonicalize().unwrap()
        );
    }
}
