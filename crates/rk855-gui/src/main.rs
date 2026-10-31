mod app;
mod state;
mod views;

use app::Rk855App;
use eframe::NativeOptions;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();
    let mock_mode = args.iter().any(|arg| arg == "--mock" || arg == "-m");

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Royal Kludge RK855 (RK68) Configurator")
            .with_inner_size([1020.0, 720.0])
            .with_min_inner_size([800.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Royal Kludge RK855",
        options,
        Box::new(move |_cc| Ok(Box::new(Rk855App::new(mock_mode)))),
    )
}
