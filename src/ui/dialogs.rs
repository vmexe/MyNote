use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use chrono::Local;
use std::path::PathBuf;
use crate::models::Note;

pub fn show_note_info_dialog(parent: &adw::ApplicationWindow, note: &Note) {
    let created_local = note.created_at.with_timezone(&Local);
    let updated_local = note.updated_at.with_timezone(&Local);

    let tags_formatted = if note.tags.is_empty() {
        "None".to_string()
    } else {
        note.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(", ")
    };

    let status_str = if note.is_trashed {
        "In Trash"
    } else if note.is_pinned {
        "Pinned ⭐"
    } else {
        "Active"
    };

    let body_text = format!(
        "Title: {}\nStatus: {}\nWords: {}\nCharacters: {}\nLines: {}\nEst. Reading Time: ~{} min\nTags: {}\nCreated: {}\nModified: {}",
        note.display_title(),
        status_str,
        note.word_count(),
        note.char_count(),
        note.line_count(),
        note.reading_time_mins(),
        tags_formatted,
        created_local.format("%Y-%m-%d %H:%M:%S"),
        updated_local.format("%Y-%m-%d %H:%M:%S")
    );

    let dialog = adw::AlertDialog::builder()
        .heading("Note Details & Statistics")
        .body(&body_text)
        .build();

    dialog.add_response("close", "Close");
    dialog.set_default_response(Some("close"));
    dialog.present(Some(parent));
}

pub fn show_confirm_empty_trash<F: FnOnce() + 'static>(parent: &adw::ApplicationWindow, on_confirm: F) {
    let dialog = adw::AlertDialog::builder()
        .heading("Empty Trash?")
        .body("All notes currently in Trash will be permanently deleted. This action cannot be undone.")
        .build();

    dialog.add_response("cancel", "Cancel");
    dialog.add_response("delete", "Empty Trash");
    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));

    let cell = std::cell::RefCell::new(Some(on_confirm));
    dialog.connect_response(None, move |_, response| {
        if response == "delete" {
            if let Some(cb) = cell.borrow_mut().take() {
                cb();
            }
        }
    });

    dialog.present(Some(parent));
}

pub fn show_confirm_delete_permanently<F: FnOnce() + 'static>(parent: &adw::ApplicationWindow, note_title: &str, on_confirm: F) {
    let dialog = adw::AlertDialog::builder()
        .heading("Delete Note Permanently?")
        .body(format!(
            "Are you sure you want to permanently delete \"{}\"? This action cannot be undone.",
            note_title
        ))
        .build();

    dialog.add_response("cancel", "Cancel");
    dialog.add_response("delete", "Delete Permanently");
    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));

    let cell = std::cell::RefCell::new(Some(on_confirm));
    dialog.connect_response(None, move |_, response| {
        if response == "delete" {
            if let Some(cb) = cell.borrow_mut().take() {
                cb();
            }
        }
    });

    dialog.present(Some(parent));
}

pub fn choose_export_file<F: FnOnce(PathBuf) + 'static>(
    parent: &adw::ApplicationWindow,
    default_name: &str,
    on_chosen: F,
) {
    let file_dialog = gtk4::FileDialog::builder()
        .title("Export Note")
        .initial_name(default_name)
        .modal(true)
        .build();

    let filter_md = gtk4::FileFilter::new();
    filter_md.set_name(Some("Markdown Files (*.md)"));
    filter_md.add_pattern("*.md");

    let filter_txt = gtk4::FileFilter::new();
    filter_txt.set_name(Some("Text Files (*.txt)"));
    filter_txt.add_pattern("*.txt");

    let filters = gio::ListStore::new::<gtk4::FileFilter>();
    filters.append(&filter_md);
    filters.append(&filter_txt);
    file_dialog.set_filters(Some(&filters));

    let cell = std::cell::RefCell::new(Some(on_chosen));
    file_dialog.save(Some(parent), gio::Cancellable::NONE, move |res| {
        if let Ok(file) = res {
            if let Some(path) = file.path() {
                if let Some(cb) = cell.borrow_mut().take() {
                    cb(path);
                }
            }
        }
    });
}

pub fn choose_import_file<F: FnOnce(PathBuf) + 'static>(
    parent: &adw::ApplicationWindow,
    on_chosen: F,
) {
    let file_dialog = gtk4::FileDialog::builder()
        .title("Import Note")
        .modal(true)
        .build();

    let filter_md = gtk4::FileFilter::new();
    filter_md.set_name(Some("Markdown & Text Files (*.md, *.txt)"));
    filter_md.add_pattern("*.md");
    filter_md.add_pattern("*.txt");
    filter_md.add_pattern("*.markdown");

    let filters = gio::ListStore::new::<gtk4::FileFilter>();
    filters.append(&filter_md);
    file_dialog.set_filters(Some(&filters));

    let cell = std::cell::RefCell::new(Some(on_chosen));
    file_dialog.open(Some(parent), gio::Cancellable::NONE, move |res| {
        if let Ok(file) = res {
            if let Some(path) = file.path() {
                if let Some(cb) = cell.borrow_mut().take() {
                    cb(path);
                }
            }
        }
    });
}

pub fn choose_backup_directory<F: FnOnce(PathBuf) + 'static>(
    parent: &adw::ApplicationWindow,
    on_chosen: F,
) {
    let file_dialog = gtk4::FileDialog::builder()
        .title("Select Backup Folder")
        .modal(true)
        .build();

    let cell = std::cell::RefCell::new(Some(on_chosen));
    file_dialog.select_folder(Some(parent), gio::Cancellable::NONE, move |res| {
        if let Ok(folder) = res {
            if let Some(path) = folder.path() {
                if let Some(cb) = cell.borrow_mut().take() {
                    cb(path);
                }
            }
        }
    });
}
