#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod ui;

fn main() -> eframe::Result {
    eframe::run_native(
        "Timestamp Cleaner",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 800.0])
                .with_min_inner_size([800.0, 600.0]),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(ui::App::default()))),
    )
}
