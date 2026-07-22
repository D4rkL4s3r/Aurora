mod app;
mod fs_ops;

use app::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Aurora",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}
