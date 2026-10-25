mod app;
mod constants;
mod icons;
mod lensing;
mod overlay;
mod physics;
mod preset;
mod render;
mod sky;
mod worker;

use eframe::egui;

fn main() {
    // Ventana grande de partida, redimensionable, con un mínimo para que los controles quepan.
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([760.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "GraviLens",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(&cc.egui_ctx)))),
    )
    .unwrap();
}
