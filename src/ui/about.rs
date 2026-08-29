use libadwaita as adw;
use libadwaita::prelude::*;

pub fn show_about_dialog(parent: &adw::ApplicationWindow) {
    let about = adw::AboutDialog::builder()
        .application_name("MyNote")
        .application_icon("org.mynote.MyNote")
        .developer_name("MyNote Open Source Contributors")
        .version("1.0.0")
        .comments("A fast, lightweight, and super easy-to-use native Linux note-taking app built with Rust, GTK4, and Libadwaita.")
        .website("https://github.com/mynote/mynote")
        .issue_url("https://github.com/mynote/mynote/issues")
        .license_type(gtk4::License::MitX11)
        .copyright("© 2026 MyNote Open Source Community")
        .developers(vec![
            "Rust & GTK4 Linux Developers".to_string(),
        ])
        .artists(vec![
            "GNOME / Adwaita Design Team".to_string(),
        ])
        .build();

    about.present(Some(parent));
}
