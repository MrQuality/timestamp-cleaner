#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod ui;

fn main() -> eframe::Result {
    let icon = image::load_from_memory_with_format(
        include_bytes!("../assets/icon-256.png"),
        image::ImageFormat::Png,
    )
    .map_err(|error| eframe::Error::AppCreation(Box::new(error)))?
    .into_rgba8();
    let (width, height) = icon.dimensions();
    eframe::run_native(
        "Timestamp Cleaner",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_icon(eframe::egui::IconData {
                    rgba: icon.into_raw(),
                    width,
                    height,
                })
                .with_inner_size([1200.0, 800.0])
                .with_min_inner_size([800.0, 600.0]),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(ui::App::default()))),
    )
}
