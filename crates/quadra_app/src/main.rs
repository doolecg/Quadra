// Entry point: opens the main window. The window opens without taking focus so
// test runs never jump in front of what doole is doing.

use eframe::egui;
use quadra_ui::MainWindow;

struct App {
    window: MainWindow,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.window.ui(ui);
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(format!("Quadra {}", env!("CARGO_PKG_VERSION")))
            .with_inner_size([1920.0, 1080.0])
            .with_min_inner_size([1024.0, 640.0])
            .with_active(false),
        ..Default::default()
    };
    eframe::run_native("Quadra", options, Box::new(|cc| Ok(Box::new(App { window: MainWindow::new(&cc.egui_ctx) }))))
}
