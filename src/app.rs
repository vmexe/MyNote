use gtk4::gdk::Display;
use gtk4::prelude::*;
use libadwaita as adw;
use crate::ui::window::MyNoteWindow;

pub const APP_ID: &str = "org.mynote.MyNote";
const STYLE_CSS: &str = include_str!("../data/style.css");

pub struct MyNoteApp {
    app: adw::Application,
}

impl MyNoteApp {
    pub fn new() -> Self {
        let app = adw::Application::builder()
            .application_id(APP_ID)
            .flags(gio::ApplicationFlags::FLAGS_NONE)
            .build();

        // The desktop launcher registers a "New Note" action that calls
        // `mynote --new-note`; honor it by starting with a fresh note.
        let start_new_note = std::env::args().any(|arg| arg == "--new-note");

        app.connect_startup(|app| {
            Self::load_css();
            Self::setup_accelerators(app);
        });

        app.connect_activate(move |app| {
            let window_handle = MyNoteWindow::new(app);
            window_handle.borrow_mut().set_start_new_note(start_new_note);
            window_handle.borrow().present();
        });

        Self { app }
    }

    fn load_css() {
        let provider = gtk4::CssProvider::new();
        provider.load_from_string(STYLE_CSS);

        if let Some(display) = Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    fn setup_accelerators(app: &adw::Application) {
        app.set_accels_for_action("win.new_note", &["<Primary>n"]);
        app.set_accels_for_action("win.search", &["<Primary>f"]);
        app.set_accels_for_action("win.save", &["<Primary>s"]);
        app.set_accels_for_action("win.toggle_pin", &["<Primary>d"]);
        app.set_accels_for_action("win.toggle_preview", &["<Primary>p"]);
        app.set_accels_for_action("win.export_markdown", &["<Primary>e"]);
        app.set_accels_for_action("win.import_note", &["<Primary>i"]);
        app.set_accels_for_action("win.delete_note", &["<Primary>Delete"]);
        app.set_accels_for_action("win.format_bold", &["<Primary>b"]);
        app.set_accels_for_action("win.format_link", &["<Primary>k"]);
        app.set_accels_for_action("win.format_code", &["<Primary><Shift>c"]);
        app.set_accels_for_action("win.shortcuts", &["<Primary>question", "F1"]);
    }

    pub fn run(&self) -> glib::ExitCode {
        self.app.run()
    }
}
