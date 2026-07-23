// Pas de console noire derrière la fenêtre en build release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod app;
mod fs_ops;
mod pane;
mod ui;

use app::App;

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Aurora")
        .with_inner_size([1100.0, 700.0]);
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../assets/aurora.png")) {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Aurora",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_theme(egui::ThemePreference::Dark);
            Ok(Box::new(App::default()))
        }),
    )
}
