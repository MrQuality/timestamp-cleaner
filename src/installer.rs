//! Installation is an explicit opt-in operation, never a side effect of discovery.
use crate::{
    Error, Result,
    dependencies::{self, DependencyReport},
    process::{self, Cancellation},
};
use std::{path::Path, process::Command, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallDecision {
    Accept,
    Decline,
}

pub fn install(decision: InstallDecision, cancel: &Cancellation) -> Result<DependencyReport> {
    if decision == InstallDecision::Decline {
        return Err(Error::Invalid("Dependency installation declined".into()));
    }
    if !cfg!(windows) {
        return Err(Error::Invalid("Automatic installation currently supports Windows. Install FFmpeg using your system package manager, then check again.".into()));
    }
    let winget = dependencies::resolve_system("winget").map_err(|_| Error::Invalid(
        "Automatic installation requires Microsoft App Installer (WinGet). Install App Installer or install FFmpeg manually, then check again.".into()))?;
    install_with(
        decision,
        &mut install_command(&winget),
        cancel,
        |command, cancel| {
            process::run(
                command,
                cancel,
                Duration::from_secs(20 * 60),
                1024 * 1024,
                None,
            )
        },
        DependencyReport::detect,
    )
}

fn install_command(winget: &Path) -> Command {
    let mut command = Command::new(winget);
    command.args([
        "install",
        "--id",
        "Gyan.FFmpeg",
        "--exact",
        "--source",
        "winget",
        "--scope",
        "user",
        "--silent",
        "--accept-package-agreements",
        "--accept-source-agreements",
        "--disable-interactivity",
    ]);
    command
}

fn install_with(
    decision: InstallDecision,
    command: &mut Command,
    cancel: &Cancellation,
    run: impl FnOnce(&mut Command, &Cancellation) -> Result<Vec<u8>>,
    detect: impl FnOnce(&Cancellation) -> DependencyReport,
) -> Result<DependencyReport> {
    if decision == InstallDecision::Decline {
        return Err(Error::Invalid("Dependency installation declined".into()));
    }
    cancel.check()?;
    run(command, cancel)?;
    cancel.check()?;
    let report = detect(cancel);
    if report.tools().is_none() {
        return Err(Error::Invalid(format!(
            "Installation finished but the dependencies are not usable:\n{}",
            report.problems().join("\n")
        )));
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependencies::Availability;
    fn ready(_: &Cancellation) -> DependencyReport {
        DependencyReport {
            ffmpeg: Availability::Ready("ffmpeg".into()),
            ffprobe: Availability::Ready("ffprobe".into()),
        }
    }
    #[test]
    fn decline_and_cancellation_never_invoke_the_package_manager() {
        let token = Cancellation::default();
        assert!(
            install_with(
                InstallDecision::Decline,
                &mut install_command(Path::new("winget")),
                &token,
                |_, _| panic!("must not install"),
                ready
            )
            .is_err()
        );
        token.cancel();
        assert!(matches!(
            install_with(
                InstallDecision::Accept,
                &mut install_command(Path::new("winget")),
                &token,
                |_, _| panic!("must not install"),
                ready
            ),
            Err(Error::Cancelled)
        ));
    }
    #[test]
    fn command_pins_identity_source_and_user_scope_without_bypassing_security_checks() {
        let command = install_command(Path::new("trusted-winget.exe"));
        let args: Vec<_> = command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            [
                "install",
                "--id",
                "Gyan.FFmpeg",
                "--exact",
                "--source",
                "winget",
                "--scope",
                "user",
                "--silent",
                "--accept-package-agreements",
                "--accept-source-agreements",
                "--disable-interactivity"
            ]
        );
    }
    #[test]
    fn successful_exit_requires_both_tools_to_be_usable() {
        let cancel = Cancellation::default();
        let result = install_with(
            InstallDecision::Accept,
            &mut install_command(Path::new("winget")),
            &cancel,
            |_, _| Ok(Vec::new()),
            |_| DependencyReport {
                ffmpeg: Availability::Ready("ffmpeg".into()),
                ffprobe: Availability::Unavailable("still missing".into()),
            },
        );
        assert!(result.is_err());
        assert!(
            install_with(
                InstallDecision::Accept,
                &mut install_command(Path::new("winget")),
                &cancel,
                |_, _| Ok(Vec::new()),
                ready
            )
            .unwrap()
            .tools()
            .is_some()
        );
    }
    #[test]
    fn failed_installation_does_not_report_success() {
        assert!(matches!(
            install_with(
                InstallDecision::Accept,
                &mut install_command(Path::new("winget")),
                &Cancellation::default(),
                |_, _| Err(Error::Timeout),
                |_| panic!("failed install must not be accepted")
            ),
            Err(Error::Timeout)
        ));
    }
}
