mod api;
mod app;
mod export;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1240.0, 820.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("Gayrimenkul360 - Ofis Portföy Yönetimi"),
        ..Default::default()
    };
    eframe::run_native(
        "Gayrimenkul360",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
