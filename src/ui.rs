use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, TextureHandle, Vec2};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use timestamp_cleaner::{
    Result,
    dependencies::DependencyReport,
    export::{self, ExportRequest},
    installer::{self, InstallDecision},
    media::{self, Media},
    model::{OutputFormat, Quality, Region},
    process::Cancellation,
};

mod dependencies;
use dependencies::{Action, DependencyState};

enum Completion {
    Dependencies(DependencyReport),
    Loaded(Media, image::RgbaImage),
    Preview(image::RgbaImage, image::RgbaImage, f64),
    Exported(PathBuf),
}
struct Job {
    handle: JoinHandle<Result<Completion>>,
    cancel: Cancellation,
    progress: Receiver<u64>,
    started: Instant,
    label: &'static str,
}

pub struct App {
    dependencies: DependencyState,
    source: Option<Media>,
    original: Option<TextureHandle>,
    repaired: Option<TextureHandle>,
    region: Region,
    drag_start: Option<Pos2>,
    quality: Quality,
    format: OutputFormat,
    seek: f64,
    shown_time: f64,
    show_repaired: bool,
    job: Option<Job>,
    progress: f32,
    status: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            dependencies: DependencyState::default(),
            source: None,
            original: None,
            repaired: None,
            region: Region {
                x: 1,
                y: 1,
                width: 2,
                height: 2,
            },
            drag_start: None,
            quality: Quality::default(),
            format: OutputFormat::default(),
            seek: 0.0,
            shown_time: 0.0,
            show_repaired: false,
            job: None,
            progress: 0.0,
            status: "Checking media dependencies…".into(),
        }
    }
}

impl App {
    fn start(
        &mut self,
        label: &'static str,
        work: impl FnOnce(Cancellation, mpsc::SyncSender<u64>) -> Result<Completion> + Send + 'static,
    ) {
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let (tx, rx) = mpsc::sync_channel(16);
        self.job = Some(Job {
            handle: thread::spawn(move || work(worker_cancel, tx)),
            cancel,
            progress: rx,
            started: Instant::now(),
            label,
        });
        self.progress = 0.0;
        self.status = label.into();
    }

    fn poll(&mut self, ctx: &egui::Context) {
        if let Some(job) = &self.job {
            for micros in job.progress.try_iter() {
                if let Some(source) = &self.source {
                    self.progress =
                        (micros as f64 / 1_000_000.0 / source.duration).clamp(0.0, 0.99) as f32;
                }
            }
            ctx.request_repaint_after(Duration::from_millis(50));
            if !job.handle.is_finished() {
                return;
            }
        } else {
            return;
        }
        let Some(job) = self.job.take() else {
            return;
        };
        let result = job.handle.join().unwrap_or_else(|_| {
            Err(timestamp_cleaner::Error::Invalid(
                "Background worker failed unexpectedly".into(),
            ))
        });
        match result {
            Ok(Completion::Dependencies(report)) => {
                self.dependencies.checked(report);
                self.status = if self.dependencies.tools().is_some() {
                    "Media tools are ready. Open a video to begin.".into()
                } else {
                    "Required media tools are unavailable. Install them or continue disabled."
                        .into()
                };
            }
            Ok(Completion::Loaded(source, frame)) => {
                self.original = Some(texture(ctx, "original", &frame));
                self.repaired = None;
                self.region = Region {
                    x: source.width / 2,
                    y: source.height * 3 / 4,
                    width: (source.width / 4).max(2),
                    height: (source.height / 10).max(2),
                };
                self.source = Some(source);
                self.seek = 0.0;
                self.shown_time = 0.0;
                self.show_repaired = false;
                self.status = "Drag over the timestamp, then preview the selected repair.".into();
            }
            Ok(Completion::Preview(original, repaired, time)) => {
                self.original = Some(texture(ctx, "original", &original));
                self.repaired = Some(texture(ctx, "repaired", &repaired));
                self.shown_time = time;
                self.show_repaired = true;
                self.status = "Preview ready. Toggle Original / Repaired to compare.".into();
            }
            Ok(Completion::Exported(path)) => {
                self.progress = 1.0;
                self.status = format!("Validated export saved to {}", path.display());
            }
            Err(error) => {
                self.dependencies.failed(&error.to_string());
                self.status = error.to_string();
            }
        }
    }

    fn dependency_action(&mut self, action: Action) {
        if self.job.is_some() {
            return;
        }
        match action {
            Action::Check => {
                self.dependencies = DependencyState::Checking;
                self.start("Checking FFmpeg and FFprobe…", |cancel, _| {
                    let report = DependencyReport::detect(&cancel);
                    cancel.check()?;
                    Ok(Completion::Dependencies(report))
                });
            }
            Action::Install if self.dependencies.begin_install() => {
                self.start("Installing FFmpeg and FFprobe…", |cancel, _| {
                    Ok(Completion::Dependencies(installer::install(
                        InstallDecision::Accept,
                        &cancel,
                    )?))
                });
            }
            Action::Decline => {
                self.dependencies.decline();
                self.status =
                    "Media operations remain disabled. You can install or check for tools later."
                        .into();
            }
            Action::Install => {}
        }
    }

    fn open(&mut self) {
        let Some(tools) = self.dependencies.tools().cloned() else {
            return;
        };
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "Videos",
                &["mp4", "mov", "mkv", "avi", "webm", "mts", "m2ts"],
            )
            .pick_file()
        {
            self.start("Opening video…", move |cancel, _| {
                let source = media::probe(&tools, &path, &cancel)?;
                let frame = media::preview(&tools, &source, 0.0, None, &cancel)?;
                Ok(Completion::Loaded(source, frame))
            });
        }
    }

    fn preview(&mut self) {
        let (Some(tools), Some(source)) = (self.dependencies.tools().cloned(), self.source.clone())
        else {
            return;
        };
        let region = self.region;
        let time = self.seek;
        self.start("Rendering preview…", move |cancel, _| {
            let original = media::preview(&tools, &source, time, None, &cancel)?;
            let repaired = media::preview(&tools, &source, time, Some(region), &cancel)?;
            Ok(Completion::Preview(original, repaired, time))
        });
    }

    fn export(&mut self) {
        let (Some(tools), Some(source)) = (self.dependencies.tools().cloned(), self.source.clone())
        else {
            return;
        };
        let name = format!(
            "{}_cleaned.{}",
            source
                .path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy(),
            self.format.extension()
        );
        if let Some(destination) = rfd::FileDialog::new()
            .set_file_name(name)
            .add_filter(self.format.label(), &[self.format.extension()])
            .save_file()
        {
            let request = ExportRequest {
                source,
                destination,
                region: self.region,
                quality: self.quality,
                format: self.format,
            };
            self.start("Exporting one selected repair…", move |cancel, tx| {
                Ok(Completion::Exported(export::export(
                    &tools,
                    &request,
                    &cancel,
                    Some(tx),
                )?))
            });
        }
    }

    fn invalidate_preview(&mut self) {
        self.repaired = None;
        self.show_repaired = false;
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Timestamp Cleaner");
        ui.label("Local video restoration");
        ui.separator();
        let busy = self.job.is_some();
        if ui
            .add_enabled(
                !busy && self.dependencies.tools().is_some(),
                egui::Button::new("Open video…"),
            )
            .clicked()
        {
            self.open();
        }
        if let Some(action) = self.dependencies.panel(ui, busy) {
            self.dependency_action(action);
        }
        let Some(source) = self.source.clone() else {
            return;
        };
        ui.label(
            source
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
        );
        ui.label(format!(
            "{} × {} · {:.1}s · {} audio stream(s)",
            source.width,
            source.height,
            source.duration,
            source.audio_codecs.len()
        ));
        ui.separator();
        ui.add_enabled_ui(!busy && self.dependencies.tools().is_some(), |ui| {
            ui.strong("Repair method");
            ui.label("Rectangle interpolation (FFmpeg)");
            ui.small("Draw over the timestamp. Keep a small context margin around it.");
            let old = self.region;
            egui::Grid::new("coordinates").show(ui, |ui| {
                for (label, value, max) in [
                    ("X", &mut self.region.x, source.width - 2),
                    ("Y", &mut self.region.y, source.height - 2),
                    ("Width", &mut self.region.width, source.width - 2),
                    ("Height", &mut self.region.height, source.height - 2),
                ] {
                    ui.label(label);
                    ui.add(egui::DragValue::new(value).range(1..=max));
                    ui.end_row();
                }
            });
            if old != self.region {
                self.invalidate_preview();
            }
            let validation = self.region.validate(source.width, source.height);
            if let Err(error) = &validation {
                ui.colored_label(Color32::LIGHT_RED, error.to_string());
            }
            ui.separator();
            ui.label("Preview position");
            let end = (source.duration - 0.1).max(0.0);
            ui.add(egui::Slider::new(&mut self.seek, 0.0..=end).suffix(" s"));
            if ui
                .add_enabled(
                    validation.is_ok(),
                    egui::Button::new("Render frame preview"),
                )
                .clicked()
            {
                self.preview();
            }
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.show_repaired, false, "Original");
                ui.add_enabled_ui(self.repaired.is_some(), |ui| {
                    ui.selectable_value(&mut self.show_repaired, true, "Repaired");
                });
            });
            ui.separator();
            egui::ComboBox::from_label("Output format")
                .selected_text(self.format.label())
                .show_ui(ui, |ui| {
                    for format in OutputFormat::ALL {
                        ui.selectable_value(&mut self.format, format, format.label());
                    }
                });
            egui::ComboBox::from_label("Export quality")
                .selected_text(self.quality.label())
                .show_ui(ui, |ui| {
                    for quality in Quality::ALL {
                        ui.selectable_value(&mut self.quality, quality, quality.label());
                    }
                });
            ui.small(match self.format {
                OutputFormat::Mkv => "H.264 / MKV · all source audio copied · source timing retained",
                OutputFormat::Mp4 => "H.264 / MP4 · AAC/MP3 copied; other audio converted to AAC · source timing retained",
            });
            if ui
                .add_enabled(
                    validation.is_ok(),
                    egui::Button::new("Export selected repair…"),
                )
                .clicked()
            {
                self.export();
            }
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        let frame = if self.show_repaired {
            self.repaired.as_ref().or(self.original.as_ref())
        } else {
            self.original.as_ref()
        };
        let (Some(frame), Some(source)) = (frame, &self.source) else {
            ui.centered_and_justified(|ui| {
                ui.label("Open a video to begin");
            });
            return;
        };
        ui.label(format!(
            "{} · frame at {:.2}s",
            if self.show_repaired {
                "Repaired"
            } else {
                "Original"
            },
            self.shown_time
        ));
        let available = ui.available_size().max(Vec2::splat(1.0));
        let ratio = (available.x / frame.size_vec2().x).min(available.y / frame.size_vec2().y);
        let response = ui.add(
            egui::Image::new((frame.id(), frame.size_vec2()))
                .fit_to_exact_size(frame.size_vec2() * ratio)
                .sense(Sense::drag()),
        );
        let bounds = response.rect;
        let width = source.width;
        let height = source.height;
        if !self.show_repaired {
            let selected = Rect::from_min_max(
                Pos2::new(
                    bounds.left() + self.region.x as f32 / width as f32 * bounds.width(),
                    bounds.top() + self.region.y as f32 / height as f32 * bounds.height(),
                ),
                Pos2::new(
                    bounds.left()
                        + (self.region.x + self.region.width) as f32 / width as f32
                            * bounds.width(),
                    bounds.top()
                        + (self.region.y + self.region.height) as f32 / height as f32
                            * bounds.height(),
                ),
            );
            ui.painter().rect_filled(
                selected,
                0.0,
                Color32::from_rgba_unmultiplied(40, 200, 170, 45),
            );
            ui.painter().rect_stroke(
                selected,
                0.0,
                Stroke::new(2.0, Color32::from_rgb(40, 220, 180)),
                StrokeKind::Inside,
            );
        }
        if self.job.is_none() && self.dependencies.tools().is_some() {
            if response.drag_started() {
                self.drag_start = response.interact_pointer_pos();
            }
            if response.dragged()
                && let (Some(start), Some(end)) = (self.drag_start, response.interact_pointer_pos())
            {
                let normalize = |p: Pos2| {
                    [
                        (p.x - bounds.left()) / bounds.width(),
                        (p.y - bounds.top()) / bounds.height(),
                    ]
                };
                if let Ok(region) =
                    Region::from_normalized_drag(normalize(start), normalize(end), width, height)
                {
                    self.region = region;
                    self.invalidate_preview();
                }
            }
            if response.drag_stopped() {
                self.drag_start = None;
            }
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if matches!(self.dependencies, DependencyState::Pending) {
            self.dependency_action(Action::Check);
        }
        self.poll(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        egui::Panel::bottom("status").show(ui, |ui| {
            ui.label(&self.status);
            if let Some(job) = &self.job {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(format!(
                        "{} · {:.0}s elapsed",
                        job.label,
                        job.started.elapsed().as_secs_f64()
                    ));
                    if ui.button("Cancel").clicked() {
                        job.cancel.cancel();
                    }
                });
                ui.add(egui::ProgressBar::new(self.progress).show_percentage());
            }
        });
        egui::Panel::left("controls")
            .default_size(290.0)
            .resizable(false)
            .show(ui, |ui| self.controls(ui));
        egui::CentralPanel::default().show(ui, |ui| self.canvas(ui));
        if let Some(action) = self.dependencies.prompt(ui.ctx()) {
            self.dependency_action(action);
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if let Some(job) = self.job.take() {
            job.cancel.cancel();
            let _ = job.handle.join();
        }
    }
}

fn texture(ctx: &egui::Context, name: &str, frame: &image::RgbaImage) -> TextureHandle {
    ctx.load_texture(
        name,
        egui::ColorImage::from_rgba_unmultiplied(
            [frame.width() as usize, frame.height() as usize],
            frame.as_raw(),
        ),
        egui::TextureOptions::LINEAR,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use timestamp_cleaner::dependencies::Tools;

    #[test]
    fn desktop_layout_handles_empty_and_loaded_states() {
        let ctx = egui::Context::default();
        let mut app = App::default();
        for (loaded, format) in [
            (false, OutputFormat::Mkv),
            (true, OutputFormat::Mkv),
            (true, OutputFormat::Mp4),
        ] {
            app.format = format;
            if loaded {
                app.dependencies = DependencyState::Ready(Tools {
                    ffmpeg: "ffmpeg".into(),
                    ffprobe: "ffprobe".into(),
                });
                app.source = Some(Media {
                    path: "family.mp4".into(),
                    width: 160,
                    height: 96,
                    duration: 1.2,
                    video_index: 0,
                    audio_codecs: vec!["aac".into()],
                });
                app.original = Some(texture(&ctx, "test", &image::RgbaImage::new(160, 96)));
            }
            let output = ctx.run_ui(egui::RawInput::default(), |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    egui::Panel::left("test-controls").show(ui, |ui| app.controls(ui));
                    app.canvas(ui);
                });
            });
            assert!(!output.shapes.is_empty());
        }
    }
}
