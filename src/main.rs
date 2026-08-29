mod app;
mod markdown;
mod models;
mod storage;
mod ui;

use app::MyNoteApp;

fn main() -> glib::ExitCode {
    let mynote = MyNoteApp::new();
    mynote.run()
}
