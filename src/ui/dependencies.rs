use eframe::egui;
use timestamp_cleaner::dependencies::{Availability, DependencyReport, Tools};

#[derive(Default)]
pub(super) enum DependencyState {
    #[default]
    Pending,
    Checking,
    Ready(Tools),
    Unavailable {
        report: DependencyReport,
        prompt: bool,
        failure: Option<String>,
    },
    Installing(DependencyReport),
}

pub(super) enum Action {
    Check,
    Install,
    Decline,
}

impl DependencyState {
    pub fn tools(&self) -> Option<&Tools> {
        match self {
            Self::Ready(tools) => Some(tools),
            _ => None,
        }
    }
    pub fn checked(&mut self, report: DependencyReport) {
        *self = match report.tools() {
            Some(tools) => Self::Ready(tools),
            None => Self::Unavailable {
                report,
                prompt: true,
                failure: None,
            },
        };
    }
    pub fn decline(&mut self) {
        if let Self::Unavailable { prompt, .. } = self {
            *prompt = false;
        }
    }
    pub fn begin_install(&mut self) -> bool {
        if let Self::Unavailable {
            report,
            prompt: true,
            ..
        } = self
        {
            *self = Self::Installing(report.clone());
            true
        } else {
            false
        }
    }
    pub fn failed(&mut self, error: &str) {
        let report = match self {
            Self::Installing(report) => report.clone(),
            Self::Checking => DependencyReport {
                ffmpeg: Availability::Unavailable("Check did not complete".into()),
                ffprobe: Availability::Unavailable("Check did not complete".into()),
            },
            _ => return,
        };
        *self = Self::Unavailable {
            report,
            prompt: false,
            failure: Some(error.into()),
        };
    }
    pub fn panel(&mut self, ui: &mut egui::Ui, busy: bool) -> Option<Action> {
        match self {
            Self::Pending | Self::Checking => {
                ui.label("Checking FFmpeg and FFprobe…");
            }
            Self::Installing(_) => {
                ui.label("Installing media tools for your Windows account…");
            }
            Self::Ready(_) => return None,
            Self::Unavailable {
                report,
                prompt,
                failure,
            } => {
                ui.strong("Media operations are disabled");
                for problem in report.problems() {
                    ui.label(problem);
                }
                if let Some(error) = failure {
                    ui.colored_label(egui::Color32::LIGHT_RED, error.as_str());
                }
                if cfg!(windows)
                    && ui
                        .add_enabled(!busy, egui::Button::new("Install missing tools…"))
                        .clicked()
                {
                    *prompt = true;
                }
                if !cfg!(windows) {
                    ui.label(
                        "Install FFmpeg through your system package manager, then check again.",
                    );
                }
                if ui
                    .add_enabled(!busy, egui::Button::new("Check again"))
                    .clicked()
                {
                    return Some(Action::Check);
                }
            }
        }
        None
    }
    pub fn prompt(&mut self, ctx: &egui::Context) -> Option<Action> {
        let Self::Unavailable {
            report,
            prompt: true,
            ..
        } = self
        else {
            return None;
        };
        let response = egui::Modal::new(egui::Id::new("install-media-dependencies")).show(ctx, |ui| {
            ui.set_max_width(460.0);
            ui.heading("Media tools are missing or unavailable");
            for problem in report.problems() { ui.label(problem); }
            ui.separator();
            if cfg!(windows) {
                ui.label("Install FFmpeg and FFprobe for your Windows account?");
                ui.label("The application will use Microsoft WinGet to download the Gyan FFmpeg package, which includes both tools. WinGet verifies the download checksum. No administrator installation is requested.");
                ui.label("Installing accepts the package and WinGet source terms.");
                ui.hyperlink_to("FFmpeg package and license details", "https://www.gyan.dev/ffmpeg/builds/");
            } else {
                ui.label("Automatic installation currently supports Windows. Install FFmpeg through your system package manager, then choose Check again.");
            }
            ui.horizontal(|ui| {
                if cfg!(windows) && ui.button("Install FFmpeg and FFprobe").clicked() { return Some(Action::Install); }
                if ui.button("Continue disabled").clicked() { return Some(Action::Decline); }
                None
            }).inner
        });
        let dismissed = response.should_close();
        response
            .inner
            .or_else(|| dismissed.then_some(Action::Decline))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn missing() -> DependencyReport {
        DependencyReport {
            ffmpeg: Availability::Ready("ffmpeg".into()),
            ffprobe: Availability::Unavailable("not found".into()),
        }
    }
    #[test]
    fn missing_declined_and_failed_states_keep_operations_disabled() {
        let mut state = DependencyState::default();
        assert!(state.tools().is_none());
        state.checked(missing());
        assert!(matches!(
            state,
            DependencyState::Unavailable { prompt: true, .. }
        ));
        assert!(state.tools().is_none());
        state.decline();
        assert!(matches!(
            state,
            DependencyState::Unavailable { prompt: false, .. }
        ));
        assert!(
            !state.begin_install(),
            "declining cannot start installation"
        );
        state.checked(missing());
        assert!(state.begin_install());
        assert!(state.tools().is_none());
        state.failed("download failed");
        assert!(matches!(
            state,
            DependencyState::Unavailable {
                prompt: false,
                failure: Some(_),
                ..
            }
        ));
        assert!(state.tools().is_none());
        let mut ready = missing();
        ready.ffprobe = Availability::Ready("ffprobe".into());
        state.checked(ready);
        assert!(state.tools().is_some());
    }

    #[test]
    fn rendering_the_prompt_never_grants_installation_consent() {
        let mut state = DependencyState::default();
        state.checked(missing());
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            assert!(state.panel(ui, false).is_none());
            assert!(state.prompt(ui.ctx()).is_none());
        });
        assert!(matches!(
            state,
            DependencyState::Unavailable { prompt: true, .. }
        ));
        state.decline();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            assert!(state.prompt(ui.ctx()).is_none());
        });
        assert!(state.tools().is_none());
    }
}
