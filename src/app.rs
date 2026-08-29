use std::cell::Cell;
use std::rc::Rc;
use gtk4::gdk::Display;
use gtk4::prelude::*;
use libadwaita as adw;
use crate::ui::window::MyNoteWindow;

pub const APP_ID: &str = "org.mynote.MyNote";
const STYLE_CSS: &str = include_str!("../data/style.css");

#[derive(Default, Clone)]
struct CliOptions {
    start_new_note: bool,
    stress_rounds: u32,
}

pub struct MyNoteApp {
    app: adw::Application,
}

impl MyNoteApp {
    pub fn new() -> Self {
        let app = adw::Application::builder()
            .application_id(APP_ID)
            .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
            .build();

        let opts = Rc::new(Cell::new(CliOptions::default()));

        app.connect_startup(|app| {
            Self::load_css();
            Self::setup_accelerators(app);
        });

        let opts_activate = opts.clone();
        app.connect_activate(move |app| {
            let opts = opts_activate.take();
            let window_handle = MyNoteWindow::new(app);

            if opts.stress_rounds > 0 {
                // Point storage at a throwaway dir so stress runs never touch
                // real notes.
                let tmp = std::env::temp_dir().join(format!("mynote_stress_{}", std::process::id()));
                std::env::set_var("MYNOTE_DATA_DIR", tmp);
            } else if opts.start_new_note {
                window_handle.borrow_mut().set_start_new_note(true);
            }

            window_handle.borrow().present();

            if opts.stress_rounds > 0 {
                MyNoteWindow::run_stress_test(window_handle, opts.stress_rounds);
            }
        });

        let opts_cl = opts.clone();
        app.connect_command_line(move |app, cmd_line| {
            let args: Vec<String> = cmd_line
                .arguments()
                .iter()
                .map(|s| s.to_string_lossy().into_owned())
                .collect();

            let mut parsed = CliOptions::default();
            let mut i = 0;
            while i < args.len() {
                let a = &args[i];
                if a == "--new-note" {
                    parsed.start_new_note = true;
                } else if a == "--stress" {
                    if let Some(n) = args.get(i + 1).and_then(|v| v.parse::<u32>().ok()) {
                        parsed.stress_rounds = n.max(4);
                        i += 1;
                    } else {
                        parsed.stress_rounds = 60;
                    }
                }
                i += 1;
            }

            opts_cl.set(parsed);
            app.activate();
            glib::ExitCode::SUCCESS
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
