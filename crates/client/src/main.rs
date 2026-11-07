mod api;
mod app;
mod export;

use eframe::egui;

/// Pencere/ görev çubuğu ikonu: assets/app-logo.png (64x64 RGBA).
fn load_app_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../../../assets/app-logo.png");
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = img.resize_exact(64, 64, image::imageops::FilterType::Lanczos3).to_rgba8();
    let (w, h) = rgba.dimensions();
    Some(egui::IconData {
        width: w,
        height: h,
        rgba: rgba.into_raw(),
    })
}

fn main() -> eframe::Result<()> {
    let icon = load_app_icon().map(std::sync::Arc::new);
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([1240.0, 820.0])
        .with_min_inner_size([900.0, 600.0])
        .with_title("İÇ SOFTWARE - Ofis Portföy Yönetimi");
    let viewport = match icon {
        Some(i) => viewport.with_icon(i),
        None => viewport,
    };
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "İÇ SOFTWARE",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
