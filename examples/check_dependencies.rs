//! Diagnose the same readiness checks used by the desktop without opening a window.
use std::process::ExitCode;
use timestamp_cleaner::{dependencies::DependencyReport, process::Cancellation};

fn main() -> ExitCode {
    let report = DependencyReport::detect(&Cancellation::default());
    if report.tools().is_some() {
        println!("FFmpeg and FFprobe are ready.");
        ExitCode::SUCCESS
    } else {
        for problem in report.problems() {
            eprintln!("{problem}");
        }
        ExitCode::FAILURE
    }
}
