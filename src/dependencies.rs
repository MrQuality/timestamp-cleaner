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
    let bytes = process::run(
        Command::new(path).arg("-version"),
        cancel,
        Duration::from_secs(10),
        64 * 1024,
        None,
    )?;
    if !bytes.starts_with(format!("{name} version ").as_bytes()) {
        return Err(Error::Invalid(format!(
            "{} did not identify itself as {name}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
