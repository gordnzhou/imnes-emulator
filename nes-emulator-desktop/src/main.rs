mod app;
mod emulator;
mod logger;
mod rom;
mod ui;

use app::App;

const TITLE: &str = "NES EMULATOR";
const WINDOW_WIDTH: u32 = 1400;
const WINDOW_HEIGHT: u32 = 800;

fn main() {
    let mut app = App::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    app.run_app(TITLE);
}