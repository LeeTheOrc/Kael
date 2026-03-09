mod ai;
mod gui;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Kael - AI Assistant",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(gui::KaelApp::new()))),
    )
}
