use std::cell::{Cell, RefCell};
use std::rc::Rc;
use chrono::Local;
use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita as adw;

use crate::markdown::markdown_to_pango;
use crate::models::{FilterMode, Note};
use crate::storage::{Settings, Storage};
use crate::ui::about::show_about_dialog;
use crate::ui::dialogs::*;
use crate::ui::shortcuts::show_shortcuts_dialog;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Edit,
    Preview,
    Split,
}

pub struct MyNoteWindow {
    window: adw::ApplicationWindow,
    toast_overlay: adw::ToastOverlay,
    _split_view: adw::NavigationSplitView,

    // Sidebar
    search_entry: gtk4::SearchEntry,
    all_notes_row: adw::ActionRow,
    all_notes_badge: gtk4::Label,
    favorites_row: adw::ActionRow,
    favorites_badge: gtk4::Label,
    trash_row: adw::ActionRow,
    trash_badge: gtk4::Label,
    tags_flowbox: gtk4::FlowBox,
    tags_section_box: gtk4::Box,
    notes_listbox: gtk4::ListBox,
    empty_trash_btn: gtk4::Button,
    new_note_btn: gtk4::Button,

    // Editor Header
    window_title: adw::WindowTitle,
    pin_btn: gtk4::ToggleButton,
    trash_btn: gtk4::Button,
    restore_btn: gtk4::Button,
    delete_perm_btn: gtk4::Button,
    view_edit_btn: gtk4::ToggleButton,
    view_preview_btn: gtk4::ToggleButton,
    view_split_btn: gtk4::ToggleButton,

    // Editor Area
    trash_banner: gtk4::Box,
    editor_container: gtk4::Box,
    empty_state_page: adw::StatusPage,
    format_toolbar: gtk4::Box,
    title_entry: gtk4::Entry,
    tags_entry: gtk4::Entry,
    text_view: gtk4::TextView,
    text_buffer: gtk4::TextBuffer,
    edit_scroll: gtk4::ScrolledWindow,
    preview_scroll: gtk4::ScrolledWindow,
    preview_label: gtk4::Label,
    split_paned: gtk4::Paned,
    content_area_box: gtk4::Box,

    // Status bar
    save_status_label: gtk4::Label,
    stats_label: gtk4::Label,

    // State
    notes: Vec<Note>,
    active_note_id: Option<String>,
    filter_mode: FilterMode,
    search_query: String,
    view_mode: ViewMode,
    is_updating_ui: Rc<Cell<bool>>,
    save_gen: Rc<Cell<u64>>,
}

pub type WindowHandle = Rc<RefCell<MyNoteWindow>>;

impl MyNoteWindow {
    pub fn new(app: &adw::Application) -> WindowHandle {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("MyNote")
            .default_width(1080)
            .default_height(720)
            .build();

        let toast_overlay = adw::ToastOverlay::new();

        // Left sidebar navigation
        let sidebar_toolbar_view = adw::ToolbarView::new();
        let sidebar_header = adw::HeaderBar::new();
        let app_title = adw::WindowTitle::new("MyNote", "");
        sidebar_header.set_title_widget(Some(&app_title));
        sidebar_toolbar_view.add_top_bar(&sidebar_header);

        // Sidebar main content
        let sidebar_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        sidebar_box.set_margin_top(8);
        sidebar_box.set_margin_bottom(8);
        sidebar_box.set_margin_start(8);
        sidebar_box.set_margin_end(8);

        // Search entry
        let search_entry = gtk4::SearchEntry::builder()
            .placeholder_text("Search notes & tags...")
            .build();
        sidebar_box.append(&search_entry);

        // Filter categories list
        let filter_listbox = gtk4::ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(["boxed-list"])
            .build();

        let all_notes_row = adw::ActionRow::builder()
            .title("All Notes")
            .activatable(true)
            .build();
        let all_icon = gtk4::Image::from_icon_name("document-edit-symbolic");
        all_notes_row.add_prefix(&all_icon);
        let all_notes_badge = gtk4::Label::new(Some("0"));
        all_notes_badge.add_css_class("filter-badge");
        all_notes_row.add_suffix(&all_notes_badge);
        filter_listbox.append(&all_notes_row);

        let favorites_row = adw::ActionRow::builder()
            .title("Favorites")
            .activatable(true)
            .build();
        let fav_icon = gtk4::Image::from_icon_name("starred-symbolic");
        favorites_row.add_prefix(&fav_icon);
        let favorites_badge = gtk4::Label::new(Some("0"));
        favorites_badge.add_css_class("filter-badge");
        favorites_row.add_suffix(&favorites_badge);
        filter_listbox.append(&favorites_row);

        let trash_row = adw::ActionRow::builder()
            .title("Trash")
            .activatable(true)
            .build();
        let trash_icon = gtk4::Image::from_icon_name("user-trash-symbolic");
        trash_row.add_prefix(&trash_icon);
        let trash_badge = gtk4::Label::new(Some("0"));
        trash_badge.add_css_class("filter-badge");
        trash_row.add_suffix(&trash_badge);
        filter_listbox.append(&trash_row);

        filter_listbox.select_row(Some(&all_notes_row));
        sidebar_box.append(&filter_listbox);

        // Tags Section
        let tags_section_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        tags_section_box.set_margin_top(4);
        let tags_header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        let tags_title = gtk4::Label::builder()
            .label("TAGS")
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .css_classes(["caption-heading"])
            .build();
        tags_header_box.append(&tags_title);
        tags_section_box.append(&tags_header_box);

        let tags_flowbox = gtk4::FlowBox::builder()
            .valign(gtk4::Align::Start)
            .max_children_per_line(6)
            .selection_mode(gtk4::SelectionMode::None)
            .build();
        tags_section_box.append(&tags_flowbox);
        sidebar_box.append(&tags_section_box);

        // Notes List Section
        let notes_label = gtk4::Label::builder()
            .label("NOTES")
            .halign(gtk4::Align::Start)
            .margin_top(8)
            .css_classes(["caption-heading"])
            .build();
        sidebar_box.append(&notes_label);

        let notes_listbox = gtk4::ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(["navigation-sidebar"])
            .build();

        let notes_scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .child(&notes_listbox)
            .build();
        sidebar_box.append(&notes_scroll);

        // Sidebar Bottom Action Bar
        let sidebar_bottom_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        sidebar_bottom_bar.set_margin_top(6);

        let new_note_btn = gtk4::Button::builder()
            .label("New Note")
            .icon_name("list-add-symbolic")
            .css_classes(["suggested-action", "pill"])
            .hexpand(true)
            .tooltip_text("Create a new note (Ctrl+N)")
            .build();
        sidebar_bottom_bar.append(&new_note_btn);

        let empty_trash_btn = gtk4::Button::builder()
            .label("Empty Trash")
            .icon_name("user-trash-full-symbolic")
            .css_classes(["destructive-action", "pill"])
            .hexpand(true)
            .visible(false)
            .tooltip_text("Permanently delete all notes in Trash")
            .build();
        sidebar_bottom_bar.append(&empty_trash_btn);

        sidebar_box.append(&sidebar_bottom_bar);
        sidebar_toolbar_view.set_content(Some(&sidebar_box));

        // Right Editor Pane
        let content_toolbar_view = adw::ToolbarView::new();
        let content_header = adw::HeaderBar::new();
        let window_title = adw::WindowTitle::new("MyNote", "");
        content_header.set_title_widget(Some(&window_title));

        // View Mode Switcher
        let view_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        view_box.add_css_class("linked");

        let view_edit_btn = gtk4::ToggleButton::builder()
            .icon_name("document-edit-symbolic")
            .tooltip_text("Edit Note")
            .active(true)
            .build();

        let view_preview_btn = gtk4::ToggleButton::builder()
            .icon_name("view-reveal-symbolic")
            .tooltip_text("Markdown Preview")
            .build();

        let view_split_btn = gtk4::ToggleButton::builder()
            .icon_name("view-dual-symbolic")
            .tooltip_text("Split Editor & Live Preview")
            .build();

        view_box.append(&view_edit_btn);
        view_box.append(&view_preview_btn);
        view_box.append(&view_split_btn);
        content_header.pack_start(&view_box);

        // Pin Button
        let pin_btn = gtk4::ToggleButton::builder()
            .icon_name("starred-symbolic")
            .tooltip_text("Toggle Favorite / Pin (Ctrl+D)")
            .build();
        content_header.pack_end(&pin_btn);

        // Trash Button
        let trash_btn = gtk4::Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text("Move to Trash (Ctrl+Delete)")
            .build();
        content_header.pack_end(&trash_btn);

        // Restore Button (for Trash filter)
        let restore_btn = gtk4::Button::builder()
            .label("Restore")
            .icon_name("edit-undo-symbolic")
            .css_classes(["suggested-action"])
            .tooltip_text("Restore note from Trash")
            .visible(false)
            .build();
        content_header.pack_end(&restore_btn);

        // Delete Permanently Button (for Trash filter)
        let delete_perm_btn = gtk4::Button::builder()
            .icon_name("edit-delete-symbolic")
            .css_classes(["destructive-action"])
            .tooltip_text("Permanently delete note")
            .visible(false)
            .build();
        content_header.pack_end(&delete_perm_btn);

        // Menu Button
        let menu_btn = gtk4::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .tooltip_text("Note Options & Actions")
            .build();

        let menu_model = gio::Menu::new();

        let note_section = gio::Menu::new();
        note_section.append(Some("Duplicate Note"), Some("win.duplicate_note"));
        note_section.append(Some("Note Details & Statistics"), Some("win.note_info"));
        menu_model.append_section(None, &note_section);

        let export_section = gio::Menu::new();
        export_section.append(Some("Export as Markdown (.md)..."), Some("win.export_markdown"));
        export_section.append(Some("Export as Plain Text (.txt)..."), Some("win.export_text"));
        export_section.append(Some("Import Note..."), Some("win.import_note"));
        export_section.append(Some("Backup All Notes..."), Some("win.backup_all"));
        menu_model.append_section(None, &export_section);

        let info_section = gio::Menu::new();
        info_section.append(Some("Keyboard Shortcuts"), Some("win.shortcuts"));
        info_section.append(Some("About MyNote"), Some("win.about"));
        menu_model.append_section(None, &info_section);

        menu_btn.set_menu_model(Some(&menu_model));
        content_header.pack_end(&menu_btn);

        content_toolbar_view.add_top_bar(&content_header);

        // Main Editor Body
        let editor_main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        // Trash warning banner
        let trash_banner = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        trash_banner.add_css_class("trash-banner");
        let trash_banner_icon = gtk4::Image::from_icon_name("user-trash-symbolic");
        let trash_banner_label = gtk4::Label::new(Some("This note is currently in Trash. Restore it to resume editing."));
        trash_banner.append(&trash_banner_icon);
        trash_banner.append(&trash_banner_label);
        trash_banner.set_visible(false);
        editor_main_box.append(&trash_banner);

        let editor_container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        editor_container.set_vexpand(true);

        // Title Entry
        let title_entry = gtk4::Entry::builder()
            .placeholder_text("Note title...")
            .css_classes(["note-title-entry"])
            .build();
        editor_container.append(&title_entry);

        // Tags Entry
        let tags_entry = gtk4::Entry::builder()
            .placeholder_text("Add tags (e.g. #work, #ideas, #todo)...")
            .css_classes(["note-tags-entry"])
            .build();
        editor_container.append(&tags_entry);

        // Formatting Toolbar
        let format_toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        format_toolbar.add_css_class("format-toolbar");

        let make_format_btn = |label: &str, tooltip: &str| -> gtk4::Button {
            gtk4::Button::builder()
                .label(label)
                .tooltip_text(tooltip)
                .css_classes(["flat", "format-btn"])
                .build()
        };

        let bold_btn = make_format_btn("B", "Bold (**text**) [Ctrl+B]");
        let italic_btn = make_format_btn("I", "Italic (*text*)");
        let strike_btn = make_format_btn("S", "Strikethrough (~~text~~)");
        let h1_btn = make_format_btn("H1", "Heading 1 (# text)");
        let h2_btn = make_format_btn("H2", "Heading 2 (## text)");
        let list_btn = make_format_btn("• List", "Bullet List (- item)");
        let task_btn = make_format_btn("☑ Task", "Task Checkbox (- [ ] task)");
        let code_btn = make_format_btn("</>", "Code Block (```)");
        let quote_btn = make_format_btn("❝ Quote", "Blockquote (> quote)");
        let link_btn = make_format_btn("🔗 Link", "Link ([title](url)) [Ctrl+K]");
        let rule_btn = make_format_btn("── Line", "Horizontal Line (---)");

        format_toolbar.append(&bold_btn);
        format_toolbar.append(&italic_btn);
        format_toolbar.append(&strike_btn);
        format_toolbar.append(&gtk4::Separator::new(gtk4::Orientation::Vertical));
        format_toolbar.append(&h1_btn);
        format_toolbar.append(&h2_btn);
        format_toolbar.append(&gtk4::Separator::new(gtk4::Orientation::Vertical));
        format_toolbar.append(&list_btn);
        format_toolbar.append(&task_btn);
        format_toolbar.append(&code_btn);
        format_toolbar.append(&quote_btn);
        format_toolbar.append(&link_btn);
        format_toolbar.append(&rule_btn);

        editor_container.append(&format_toolbar);

        // Content Area (Houses Editor ScrolledWindow, Preview ScrolledWindow, or Paned)
        let content_area_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        content_area_box.set_vexpand(true);

        // 1. Text Editor
        let text_buffer = gtk4::TextBuffer::new(None);
        let text_view = gtk4::TextView::builder()
            .buffer(&text_buffer)
            .wrap_mode(gtk4::WrapMode::WordChar)
            .monospace(false)
            .top_margin(12)
            .bottom_margin(12)
            .left_margin(16)
            .right_margin(16)
            .css_classes(["note-text-view"])
            .vexpand(true)
            .build();

        let edit_scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&text_view)
            .vexpand(true)
            .build();

        // 2. Preview View
        let preview_label = gtk4::Label::builder()
            .wrap(true)
            .wrap_mode(pango::WrapMode::WordChar)
            .use_markup(true)
            .selectable(true)
            .xalign(0.0)
            .yalign(0.0)
            .css_classes(["note-preview-label"])
            .build();

        let preview_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        preview_box.add_css_class("note-preview-box");
        preview_box.append(&preview_label);

        let preview_scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&preview_box)
            .vexpand(true)
            .build();

        // 3. Split Paned
        let split_paned = gtk4::Paned::new(gtk4::Orientation::Horizontal);
        split_paned.set_vexpand(true);

        // Initial setup: Edit view
        content_area_box.append(&edit_scroll);
        editor_container.append(&content_area_box);

        // Status bar
        let status_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        status_bar.add_css_class("note-status-bar");

        let save_status_label = gtk4::Label::builder()
            .label("Saved")
            .halign(gtk4::Align::Start)
            .build();
        status_bar.append(&save_status_label);

        let stats_label = gtk4::Label::builder()
            .label("0 words | 0 characters")
            .halign(gtk4::Align::End)
            .hexpand(true)
            .build();
        status_bar.append(&stats_label);

        editor_container.append(&status_bar);
        editor_main_box.append(&editor_container);

        // Empty state page (when no note is selected or list is empty)
        let empty_state_page = adw::StatusPage::builder()
            .icon_name("document-new-symbolic")
            .title("No Note Selected")
            .description("Select a note from the sidebar or create a new one.")
            .vexpand(true)
            .visible(false)
            .build();
        editor_main_box.append(&empty_state_page);

        content_toolbar_view.set_content(Some(&editor_main_box));

        // Navigation Split View layout
        let split_view = adw::NavigationSplitView::new();
        split_view.set_sidebar(Some(&adw::NavigationPage::new(&sidebar_toolbar_view, "Sidebar")));
        split_view.set_content(Some(&adw::NavigationPage::new(&content_toolbar_view, "Content")));
        split_view.set_min_sidebar_width(280.0);
        split_view.set_max_sidebar_width(380.0);

        toast_overlay.set_child(Some(&split_view));
        window.set_content(Some(&toast_overlay));

        // Load saved notes and settings
        let notes = Storage::load_notes();
        let settings = Storage::load_settings();

        let initial_view_mode = match settings.view_mode.as_deref() {
            Some("preview") => ViewMode::Preview,
            Some("split") => ViewMode::Split,
            _ => ViewMode::Edit,
        };

        // Prefer the previously active note if it still exists, otherwise fall
        // back to the first non-trashed note.
        let initial_note_id = notes
            .iter()
            .find(|n| settings.last_note_id.as_deref().is_some_and(|id| n.id == id) && !n.is_trashed)
            .map(|n| n.id.clone())
            .or_else(|| notes.iter().find(|n| !n.is_trashed).map(|n| n.id.clone()));

        let window_handle = Rc::new(RefCell::new(Self {
            window,
            toast_overlay,
            _split_view: split_view,
            search_entry,
            all_notes_row,
            all_notes_badge,
            favorites_row,
            favorites_badge,
            trash_row,
            trash_badge,
            tags_flowbox,
            tags_section_box,
            notes_listbox,
            empty_trash_btn,
            new_note_btn,
            window_title,
            pin_btn,
            trash_btn,
            restore_btn,
            delete_perm_btn,
            view_edit_btn,
            view_preview_btn,
            view_split_btn,
            trash_banner,
            editor_container,
            empty_state_page,
            format_toolbar,
            title_entry,
            tags_entry,
            text_view,
            text_buffer,
            edit_scroll,
            preview_scroll,
            preview_label,
            split_paned,
            content_area_box,
            save_status_label,
            stats_label,
            notes,
            active_note_id: initial_note_id,
            filter_mode: FilterMode::All,
            search_query: String::new(),
            view_mode: initial_view_mode,
            is_updating_ui: Rc::new(Cell::new(false)),
            save_gen: Rc::new(Cell::new(0)),
        }));

        // Set up formatting actions
        {
            let handle = window_handle.clone();
            bold_btn.connect_clicked(move |_| {
                handle.borrow().insert_format("**", "**", "bold text");
            });
        }
        {
            let handle = window_handle.clone();
            italic_btn.connect_clicked(move |_| {
                handle.borrow().insert_format("*", "*", "italic text");
            });
        }
        {
            let handle = window_handle.clone();
            strike_btn.connect_clicked(move |_| {
                handle.borrow().insert_format("~~", "~~", "strikethrough");
            });
        }
        {
            let handle = window_handle.clone();
            h1_btn.connect_clicked(move |_| {
                handle.borrow().insert_line_prefix("# ");
            });
        }
        {
            let handle = window_handle.clone();
            h2_btn.connect_clicked(move |_| {
                handle.borrow().insert_line_prefix("## ");
            });
        }
        {
            let handle = window_handle.clone();
            list_btn.connect_clicked(move |_| {
                handle.borrow().insert_line_prefix("- ");
            });
        }
        {
            let handle = window_handle.clone();
            task_btn.connect_clicked(move |_| {
                handle.borrow().insert_line_prefix("- [ ] ");
            });
        }
        {
            let handle = window_handle.clone();
            code_btn.connect_clicked(move |_| {
                handle.borrow().insert_format("```\n", "\n```", "code block");
            });
        }
        {
            let handle = window_handle.clone();
            quote_btn.connect_clicked(move |_| {
                handle.borrow().insert_line_prefix("> ");
            });
        }
        {
            let handle = window_handle.clone();
            link_btn.connect_clicked(move |_| {
                handle.borrow().insert_format("[", "](https://example.com)", "link title");
            });
        }
        {
            let handle = window_handle.clone();
            rule_btn.connect_clicked(move |_| {
                handle.borrow().insert_text_at_cursor("\n\n---\n\n");
            });
        }

        // Set up interactions and signal connections
        Self::setup_signals(window_handle.clone());
        Self::setup_actions(window_handle.clone());

        // Initial UI refresh. The active note id and view mode were derived
        // from saved settings above.
        window_handle.borrow_mut().set_view_mode(initial_view_mode);
        window_handle.borrow_mut().refresh_sidebar();
        window_handle.borrow_mut().refresh_editor();

        // Persist the active note and view mode when the window is closed.
        {
            let h2 = window_handle.clone();
            window_handle.borrow().window.connect_close_request(move |_| {
                let state = h2.borrow();
                let settings = Settings {
                    last_note_id: state.active_note_id.clone(),
                    view_mode: Some(match state.view_mode {
                        ViewMode::Edit => "edit".to_string(),
                        ViewMode::Preview => "preview".to_string(),
                        ViewMode::Split => "split".to_string(),
                    }),
                };
                drop(state);
                let _ = Storage::save_settings(&settings);
                glib::Propagation::Proceed
            });
        }

        window_handle
    }

    pub fn present(&self) {
        self.window.present();
    }

    pub fn set_start_new_note(&mut self, enabled: bool) {
        if enabled {
            self.create_new_note();
        }
    }

    /// Stress-test driver. Drives every action and widget signal path in a
    /// loop so that panics/crashes (double borrows, stale state, etc.) surface
    /// during automated testing. Widgets are cloned out of the handle first so
    /// their signals fire like real user events (no outstanding borrow).
    pub fn run_stress_test(handle: WindowHandle, rounds: u32) {
        Self::stress_round(handle, rounds, 0);
    }

    fn stress_round(handle: WindowHandle, total: u32, done: u32) {
        if done >= total {
            handle.borrow().window.close();
            eprintln!("[stress] completed {} rounds without crash", done);
            return;
        }

        // Round 1: widget-driven (fires real signals)
        let (pin, trash_btn, restore_btn, del_btn, ve, vp, vs) = {
            let s = handle.borrow();
            (
                s.pin_btn.clone(),
                s.trash_btn.clone(),
                s.restore_btn.clone(),
                s.delete_perm_btn.clone(),
                s.view_edit_btn.clone(),
                s.view_preview_btn.clone(),
                s.view_split_btn.clone(),
            )
        };
        pin.set_active(!pin.is_active());
        vp.set_active(true);
        ve.set_active(true);
        vs.set_active(true);
        ve.set_active(true);

        // Round 2: action-driven (routes through same handlers as shortcuts)
        let win = handle.borrow().window.clone();
        use gio::prelude::ActionGroupExt;
        ActionGroupExt::activate_action(&win, "new_note", None);
        ActionGroupExt::activate_action(&win, "toggle_pin", None);
        ActionGroupExt::activate_action(&win, "toggle_preview", None);
        ActionGroupExt::activate_action(&win, "save", None);
        ActionGroupExt::activate_action(&win, "duplicate_note", None);
        ActionGroupExt::activate_action(&win, "format_bold", None);
        if done.is_multiple_of(3) {
            trash_btn.emit_clicked();
        } else if done % 3 == 1 {
            restore_btn.emit_clicked();
        } else {
            trash_btn.emit_clicked();
        }
        let _ = del_btn;

        // Round 3: logic paths directly against a stable clone
        {
            let mut s = handle.borrow_mut();
            if s.active_note().is_some() {
                s.duplicate_active_note();
            }
            s.on_title_changed(format!("Stress note {}", done));
            s.on_tags_changed("#work, #test, #stress".to_string());
            s.on_content_changed(format!(
                "# Heading {}\n\n- [x] done\n- [ ] todo\n\n**bold** *italic* `code`\n\n```rust\nfn main() {{}}\n```\n\n> quote\n\n---\n",
                done
            ));
            s.save_immediately();
            s.schedule_save();
        }

        // Toggle filters through sidebar rows
        {
            let (all, fav, trash_row) = {
                let s = handle.borrow();
                (s.all_notes_row.clone(), s.favorites_row.clone(), s.trash_row.clone())
            };
            use libadwaita::prelude::ActionRowExt;
            match done % 4 {
                0 => ActionRowExt::activate(&all),
                1 => ActionRowExt::activate(&fav),
                2 => ActionRowExt::activate(&trash_row),
                _ => ActionRowExt::activate(&all),
            }
        }

        // Search
        {
            let search = {
                let s = handle.borrow();
                s.search_entry.clone()
            };
            match done % 3 {
                0 => search.set_text("work"),
                1 => search.set_text("nomatchxyz"),
                _ => search.set_text(""),
            }
        }

        glib::timeout_add_local_once(std::time::Duration::from_millis(4), move || {
            Self::stress_round(handle, total, done + 1);
        });
    }

    fn setup_signals(handle: WindowHandle) {
        // Search Entry
        {
            let h = handle.clone();
            handle.borrow().search_entry.connect_search_changed(move |entry| {
                let query = entry.text().to_string();
                h.borrow_mut().search_query = query;
                h.borrow_mut().refresh_sidebar();
            });
        }

        // Filter Category Rows
        {
            let h = handle.clone();
            handle.borrow().all_notes_row.connect_activated(move |_| {
                h.borrow_mut().filter_mode = FilterMode::All;
                h.borrow_mut().refresh_sidebar();
                h.borrow_mut().select_first_visible_note();
            });
        }
        {
            let h = handle.clone();
            handle.borrow().favorites_row.connect_activated(move |_| {
                h.borrow_mut().filter_mode = FilterMode::Favorites;
                h.borrow_mut().refresh_sidebar();
                h.borrow_mut().select_first_visible_note();
            });
        }
        {
            let h = handle.clone();
            handle.borrow().trash_row.connect_activated(move |_| {
                h.borrow_mut().filter_mode = FilterMode::Trash;
                h.borrow_mut().refresh_sidebar();
                h.borrow_mut().select_first_visible_note();
            });
        }

        // Tags Flowbox Child Activated
        {
            let h = handle.clone();
            handle.borrow().tags_flowbox.connect_child_activated(move |_, child| {
                if let Some(btn) = child.child().and_downcast::<gtk4::Button>() {
                    let tag = btn.widget_name().to_string();
                    if !tag.is_empty() {
                        let is_current = match h.borrow().filter_mode {
                            FilterMode::Tag(ref t) => t == &tag,
                            _ => false,
                        };
                        if is_current {
                            h.borrow_mut().filter_mode = FilterMode::All;
                        } else {
                            h.borrow_mut().filter_mode = FilterMode::Tag(tag);
                        }
                        h.borrow_mut().refresh_sidebar();
                        h.borrow_mut().select_first_visible_note();
                    }
                }
            });
        }

        // New Note Button
        {
            let h = handle.clone();
            handle.borrow().new_note_btn.connect_clicked(move |_| {
                h.borrow_mut().create_new_note();
            });
        }

        // Empty Trash Button
        {
            let h = handle.clone();
            handle.borrow().empty_trash_btn.connect_clicked(move |_| {
                let win = h.borrow().window.clone();
                let h_inner = h.clone();
                show_confirm_empty_trash(&win, move || {
                    h_inner.borrow_mut().empty_trash();
                });
            });
        }

        // Pin Toggle Button
        {
            let h = handle.clone();
            let guard = handle.borrow().is_updating_ui.clone();
            handle.borrow().pin_btn.connect_toggled(move |btn| {
                if guard.get() {
                    return;
                }
                let active = btn.is_active();
                h.borrow_mut().set_active_note_pinned(active);
            });
        }

        // Trash Button
        {
            let h = handle.clone();
            handle.borrow().trash_btn.connect_clicked(move |_| {
                h.borrow_mut().move_active_note_to_trash();
            });
        }

        // Restore Button
        {
            let h = handle.clone();
            handle.borrow().restore_btn.connect_clicked(move |_| {
                h.borrow_mut().restore_active_note();
            });
        }

        // Delete Permanently Button
        {
            let h = handle.clone();
            handle.borrow().delete_perm_btn.connect_clicked(move |_| {
                let title = h.borrow().active_note().map(|n| n.display_title()).unwrap_or_default();
                let win = h.borrow().window.clone();
                let h_inner = h.clone();
                show_confirm_delete_permanently(&win, &title, move || {
                    h_inner.borrow_mut().delete_active_note_permanently();
                });
            });
        }

        // View Mode Toggle Buttons
        {
            let h = handle.clone();
            let guard = handle.borrow().is_updating_ui.clone();
            handle.borrow().view_edit_btn.connect_toggled(move |btn| {
                if guard.get() {
                    return;
                }
                if btn.is_active() {
                    h.borrow_mut().set_view_mode(ViewMode::Edit);
                }
            });
        }
        {
            let h = handle.clone();
            let guard = handle.borrow().is_updating_ui.clone();
            handle.borrow().view_preview_btn.connect_toggled(move |btn| {
                if guard.get() {
                    return;
                }
                if btn.is_active() {
                    h.borrow_mut().set_view_mode(ViewMode::Preview);
                }
            });
        }
        {
            let h = handle.clone();
            let guard = handle.borrow().is_updating_ui.clone();
            handle.borrow().view_split_btn.connect_toggled(move |btn| {
                if guard.get() {
                    return;
                }
                if btn.is_active() {
                    h.borrow_mut().set_view_mode(ViewMode::Split);
                }
            });
        }

        // Note Title Entry Edited
        {
            let h = handle.clone();
            let guard = handle.borrow().is_updating_ui.clone();
            handle.borrow().title_entry.connect_changed(move |entry| {
                if guard.get() {
                    return;
                }
                let title = entry.text().to_string();
                h.borrow_mut().on_title_changed(title);
            });
        }

        // Note Tags Entry Edited
        {
            let h = handle.clone();
            let guard = handle.borrow().is_updating_ui.clone();
            handle.borrow().tags_entry.connect_changed(move |entry| {
                if guard.get() {
                    return;
                }
                let tags_str = entry.text().to_string();
                h.borrow_mut().on_tags_changed(tags_str);
            });
        }

        // TextBuffer Content Edited
        {
            let h = handle.clone();
            let guard = handle.borrow().is_updating_ui.clone();
            handle.borrow().text_buffer.connect_changed(move |buf| {
                if guard.get() {
                    return;
                }
                let start = buf.start_iter();
                let end = buf.end_iter();
                let content = buf.text(&start, &end, false).to_string();
                h.borrow_mut().on_content_changed(content);
            });
        }

        // Notes List Row Selected
        {
            let h = handle.clone();
            let guard = handle.borrow().is_updating_ui.clone();
            handle.borrow().notes_listbox.connect_row_selected(move |_, row_opt| {
                if guard.get() {
                    return;
                }
                if let Some(row) = row_opt {
                    let note_id = row.widget_name().to_string();
                    if !note_id.is_empty() {
                        h.borrow_mut().select_note(note_id);
                    }
                }
            });
        }
    }

    fn setup_actions(handle: WindowHandle) {
        let window = handle.borrow().window.clone();

        // win.new_note
        let act_new = gio::SimpleAction::new("new_note", None);
        {
            let h = handle.clone();
            act_new.connect_activate(move |_, _| {
                h.borrow_mut().create_new_note();
            });
        }
        window.add_action(&act_new);

        // win.search
        let act_search = gio::SimpleAction::new("search", None);
        {
            let h = handle.clone();
            act_search.connect_activate(move |_, _| {
                h.borrow().search_entry.grab_focus();
            });
        }
        window.add_action(&act_search);

        // win.save
        let act_save = gio::SimpleAction::new("save", None);
        {
            let h = handle.clone();
            act_save.connect_activate(move |_, _| {
                h.borrow_mut().save_immediately();
            });
        }
        window.add_action(&act_save);

        // win.format_bold
        let act_bold = gio::SimpleAction::new("format_bold", None);
        {
            let h = handle.clone();
            act_bold.connect_activate(move |_, _| {
                h.borrow_mut().with_ui_guard(|win| win.insert_format("**", "**", "bold text"));
            });
        }
        window.add_action(&act_bold);

        // win.format_link
        let act_link = gio::SimpleAction::new("format_link", None);
        {
            let h = handle.clone();
            act_link.connect_activate(move |_, _| {
                h.borrow_mut().with_ui_guard(|win| win.insert_format("[", "](https://example.com)", "link title"));
            });
        }
        window.add_action(&act_link);

        // win.format_code
        let act_code = gio::SimpleAction::new("format_code", None);
        {
            let h = handle.clone();
            act_code.connect_activate(move |_, _| {
                h.borrow_mut().with_ui_guard(|win| win.insert_format("```\n", "\n```", "code block"));
            });
        }
        window.add_action(&act_code);

        // win.toggle_pin
        let act_pin = gio::SimpleAction::new("toggle_pin", None);
        {
            let h = handle.clone();
            act_pin.connect_activate(move |_, _| {
                let pin_btn = h.borrow().pin_btn.clone();
                pin_btn.set_active(!pin_btn.is_active());
            });
        }
        window.add_action(&act_pin);

        // win.toggle_preview
        let act_prev = gio::SimpleAction::new("toggle_preview", None);
        {
            let h = handle.clone();
            act_prev.connect_activate(move |_, _| {
                let next_mode = match h.borrow().view_mode {
                    ViewMode::Edit => ViewMode::Preview,
                    ViewMode::Preview => ViewMode::Split,
                    ViewMode::Split => ViewMode::Edit,
                };
                h.borrow_mut().set_view_mode(next_mode);
            });
        }
        window.add_action(&act_prev);

        // win.delete_note
        let act_del = gio::SimpleAction::new("delete_note", None);
        {
            let h = handle.clone();
            act_del.connect_activate(move |_, _| {
                if h.borrow().filter_mode == FilterMode::Trash {
                    let title = h.borrow().active_note().map(|n| n.display_title()).unwrap_or_default();
                    let win = h.borrow().window.clone();
                    let h_inner = h.clone();
                    show_confirm_delete_permanently(&win, &title, move || {
                        h_inner.borrow_mut().delete_active_note_permanently();
                    });
                } else {
                    h.borrow_mut().move_active_note_to_trash();
                }
            });
        }
        window.add_action(&act_del);

        // win.export_markdown
        let act_exp_md = gio::SimpleAction::new("export_markdown", None);
        {
            let h = handle.clone();
            act_exp_md.connect_activate(move |_, _| {
                h.borrow_mut().export_active_note_dialog(false);
            });
        }
        window.add_action(&act_exp_md);

        // win.export_text
        let act_exp_txt = gio::SimpleAction::new("export_text", None);
        {
            let h = handle.clone();
            act_exp_txt.connect_activate(move |_, _| {
                h.borrow_mut().export_active_note_dialog(true);
            });
        }
        window.add_action(&act_exp_txt);

        // win.import_note
        let act_imp = gio::SimpleAction::new("import_note", None);
        {
            let h = handle.clone();
            act_imp.connect_activate(move |_, _| {
                Self::import_note_dialog(h.clone());
            });
        }
        window.add_action(&act_imp);

        // win.backup_all
        let act_backup = gio::SimpleAction::new("backup_all", None);
        {
            let h = handle.clone();
            act_backup.connect_activate(move |_, _| {
                h.borrow_mut().backup_all_notes_dialog();
            });
        }
        window.add_action(&act_backup);

        // win.duplicate_note
        let act_dup = gio::SimpleAction::new("duplicate_note", None);
        {
            let h = handle.clone();
            act_dup.connect_activate(move |_, _| {
                h.borrow_mut().duplicate_active_note();
            });
        }
        window.add_action(&act_dup);

        // win.note_info
        let act_info = gio::SimpleAction::new("note_info", None);
        {
            let h = handle.clone();
            act_info.connect_activate(move |_, _| {
                if let Some(note) = h.borrow().active_note() {
                    show_note_info_dialog(&h.borrow().window, &note);
                }
            });
        }
        window.add_action(&act_info);

        // win.shortcuts
        let act_short = gio::SimpleAction::new("shortcuts", None);
        {
            let h = handle.clone();
            act_short.connect_activate(move |_, _| {
                show_shortcuts_dialog(&h.borrow().window);
            });
        }
        window.add_action(&act_short);

        // win.about
        let act_about = gio::SimpleAction::new("about", None);
        {
            let h = handle.clone();
            act_about.connect_activate(move |_, _| {
                show_about_dialog(&h.borrow().window);
            });
        }
        window.add_action(&act_about);
    }

    pub fn active_note(&self) -> Option<Note> {
        let active_id = self.active_note_id.as_ref()?;
        self.notes.iter().find(|n| &n.id == active_id).cloned()
    }

    fn with_ui_guard<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.is_updating_ui.set(true);
        let result = f(self);
        self.is_updating_ui.set(false);
        result
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.is_updating_ui.set(true);
        self.view_mode = mode;
        self.view_edit_btn.set_active(mode == ViewMode::Edit);
        self.view_preview_btn.set_active(mode == ViewMode::Preview);
        self.view_split_btn.set_active(mode == ViewMode::Split);

        // Clear content area box
        while let Some(child) = self.content_area_box.first_child() {
            self.content_area_box.remove(&child);
        }

        // Clean split paned children
        self.split_paned.set_start_child(gtk4::Widget::NONE);
        self.split_paned.set_end_child(gtk4::Widget::NONE);

        match mode {
            ViewMode::Edit => {
                self.content_area_box.append(&self.edit_scroll);
                self.format_toolbar.set_visible(true);
            }
            ViewMode::Preview => {
                self.content_area_box.append(&self.preview_scroll);
                self.format_toolbar.set_visible(false);
                self.update_preview_content();
            }
            ViewMode::Split => {
                self.split_paned.set_start_child(Some(&self.edit_scroll));
                self.split_paned.set_end_child(Some(&self.preview_scroll));
                self.split_paned.set_position(500);
                self.content_area_box.append(&self.split_paned);
                self.format_toolbar.set_visible(true);
                self.update_preview_content();
            }
        }

        self.is_updating_ui.set(false);
    }

    pub fn create_new_note(&mut self) {
        let mut note = Note::new();
        if let FilterMode::Tag(ref tag) = self.filter_mode {
            note.tags.push(tag.clone());
        }
        let note_id = note.id.clone();
        self.notes.insert(0, note);
        self.active_note_id = Some(note_id);

        self.refresh_sidebar();
        self.refresh_editor();
        self.title_entry.grab_focus();
        self.schedule_save();
    }

    pub fn duplicate_active_note(&mut self) {
        let Some(original) = self.active_note() else { return; };
        let mut copy = Note::new();
        copy.title = format!("{} (copy)", original.title.trim());
        copy.content = original.content.clone();
        copy.tags = original.tags.clone();
        copy.is_pinned = false;

        let copy_id = copy.id.clone();
        self.notes.insert(0, copy);
        self.active_note_id = Some(copy_id);

        self.refresh_sidebar();
        self.refresh_editor();
        self.schedule_save();

        let toast = adw::Toast::new("Note duplicated");
        self.toast_overlay.add_toast(toast);
    }

    pub fn select_note(&mut self, id: String) {
        if self.active_note_id.as_ref() == Some(&id) {
            return;
        }
        self.active_note_id = Some(id);
        self.refresh_editor();
    }

    pub fn select_first_visible_note(&mut self) {
        let first_id = self.visible_notes().first().map(|n| n.id.clone());
        self.active_note_id = first_id;
        self.refresh_editor();
    }

    pub fn visible_notes(&self) -> Vec<&Note> {
        let mut list: Vec<&Note> = self.notes.iter()
            .filter(|n| self.filter_mode.matches(n))
            .filter(|n| n.matches_query(&self.search_query))
            .collect();

        // Sort: Pinned first, then by updated_at descending
        list.sort_by(|a, b| {
            b.is_pinned.cmp(&a.is_pinned)
                .then_with(|| b.updated_at.cmp(&a.updated_at))
        });

        list
    }

    pub fn set_active_note_pinned(&mut self, pinned: bool) {
        let Some(active_id) = self.active_note_id.clone() else { return; };
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == active_id) {
            note.is_pinned = pinned;
            note.touch();
        }
        self.refresh_sidebar();
        self.schedule_save();
    }

    pub fn move_active_note_to_trash(&mut self) {
        let Some(active_id) = self.active_note_id.clone() else { return; };
        let mut note_title = String::new();
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == active_id) {
            note.is_trashed = true;
            note.touch();
            note_title = note.display_title();
        }

        // Show toast
        let toast = adw::Toast::new(&format!("Moved \"{}\" to Trash", note_title));
        self.toast_overlay.add_toast(toast);

        self.select_first_visible_note();
        self.refresh_sidebar();
        self.schedule_save();
    }

    pub fn restore_active_note(&mut self) {
        let Some(active_id) = self.active_note_id.clone() else { return; };
        let mut note_title = String::new();
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == active_id) {
            note.is_trashed = false;
            note.touch();
            note_title = note.display_title();
        }

        let toast = adw::Toast::new(&format!("Restored \"{}\"", note_title));
        self.toast_overlay.add_toast(toast);

        self.refresh_sidebar();
        self.refresh_editor();
        self.schedule_save();
    }

    pub fn delete_active_note_permanently(&mut self) {
        let Some(active_id) = self.active_note_id.clone() else { return; };
        self.notes.retain(|n| n.id != active_id);

        let toast = adw::Toast::new("Note permanently deleted");
        self.toast_overlay.add_toast(toast);

        self.select_first_visible_note();
        self.refresh_sidebar();
        self.refresh_editor();
        self.schedule_save();
    }

    pub fn empty_trash(&mut self) {
        let count = self.notes.iter().filter(|n| n.is_trashed).count();
        self.notes.retain(|n| !n.is_trashed);

        let toast = adw::Toast::new(&format!("Trash emptied ({} notes deleted)", count));
        self.toast_overlay.add_toast(toast);

        self.select_first_visible_note();
        self.refresh_sidebar();
        self.refresh_editor();
        self.schedule_save();
    }

    pub fn on_title_changed(&mut self, title: String) {
        let Some(active_id) = self.active_note_id.clone() else { return; };
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == active_id) {
            note.title = title;
            note.touch();
        }
        self.window_title.set_title(&self.active_note().map(|n| n.display_title()).unwrap_or_default());
        self.refresh_sidebar_note_items();
        self.schedule_save();
    }

    pub fn on_tags_changed(&mut self, tags_str: String) {
        let Some(active_id) = self.active_note_id.clone() else { return; };
        let tags: Vec<String> = tags_str
            .split([',', ' '])
            .map(|t| t.trim().trim_start_matches('#').to_string())
            .filter(|t| !t.is_empty())
            .collect();

        if let Some(note) = self.notes.iter_mut().find(|n| n.id == active_id) {
            note.tags = tags;
            note.touch();
        }
        self.refresh_sidebar_note_items();
        self.refresh_tags_flowbox();
        self.schedule_save();
    }

    pub fn on_content_changed(&mut self, content: String) {
        let Some(active_id) = self.active_note_id.clone() else { return; };
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == active_id) {
            note.content = content;
            note.touch();
        }
        self.update_stats();
        if self.view_mode != ViewMode::Edit {
            self.update_preview_content();
        }
        self.refresh_sidebar_note_items();
        self.schedule_save();
    }

    pub fn insert_format(&self, prefix: &str, suffix: &str, placeholder: &str) {
        if let Some((start, end)) = self.text_buffer.selection_bounds() {
            let selected_text = self.text_buffer.text(&start, &end, false);
            let replacement = format!("{}{}{}", prefix, selected_text, suffix);
            self.text_buffer.delete(&mut start.clone(), &mut end.clone());
            self.text_buffer.insert(&mut start.clone(), &replacement);
        } else {
            let insert_mark = self.text_buffer.get_insert();
            let mut insert_iter = self.text_buffer.iter_at_mark(&insert_mark);
            let replacement = format!("{}{}{}", prefix, placeholder, suffix);
            self.text_buffer.insert(&mut insert_iter, &replacement);
        }
        self.text_view.grab_focus();
    }

    pub fn insert_line_prefix(&self, prefix: &str) {
        let insert_mark = self.text_buffer.get_insert();
        let mut insert_iter = self.text_buffer.iter_at_mark(&insert_mark);
        insert_iter.backward_line();
        self.text_buffer.insert(&mut insert_iter, prefix);
        self.text_view.grab_focus();
    }

    pub fn insert_text_at_cursor(&self, text: &str) {
        let insert_mark = self.text_buffer.get_insert();
        let mut insert_iter = self.text_buffer.iter_at_mark(&insert_mark);
        self.text_buffer.insert(&mut insert_iter, text);
        self.text_view.grab_focus();
    }

    pub fn schedule_save(&mut self) {
        self.save_status_label.set_label("Saving...");

        // Generation-based debounce. A new generation invalidates any still
        // pending (not yet fired) save. We never call SourceId::remove() on a
        // potentially already-fired timeout, which would panic.
        let gen = self.save_gen.get().wrapping_add(1);
        self.save_gen.set(gen);

        let notes_to_save = self.notes.clone();
        let status_label = self.save_status_label.clone();
        let gen_ref = self.save_gen.clone();

        glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
            // A more recent save was scheduled after us; let it win.
            if gen_ref.get() != gen {
                return;
            }
            if let Err(e) = Storage::save_notes(&notes_to_save) {
                status_label.set_label(&format!("Save error: {}", e));
            } else {
                let now_str = Local::now().format("%H:%M").to_string();
                status_label.set_label(&format!("Saved at {}", now_str));
            }
        });
    }

    pub fn save_immediately(&mut self) {
        // Bump the generation so any pending debounced save is skipped.
        self.save_gen.set(self.save_gen.get().wrapping_add(1));

        if let Err(e) = Storage::save_notes(&self.notes) {
            self.save_status_label.set_label(&format!("Save error: {}", e));
        } else {
            let now_str = Local::now().format("%H:%M:%S").to_string();
            self.save_status_label.set_label(&format!("Saved at {}", now_str));
            let toast = adw::Toast::new("Note saved successfully");
            self.toast_overlay.add_toast(toast);
        }
    }

    pub fn export_active_note_dialog(&self, is_plain_text: bool) {
        let Some(note) = self.active_note() else { return; };
        let default_name = format!(
            "{}.{}",
            note.display_title().replace(' ', "_").to_lowercase(),
            if is_plain_text { "txt" } else { "md" }
        );

        let note_clone = note.clone();
        let toast_overlay = self.toast_overlay.clone();

        choose_export_file(&self.window, &default_name, move |path| {
            match Storage::export_note_to_file(&note_clone, &path) {
                Ok(_) => {
                    let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("file");
                    toast_overlay.add_toast(adw::Toast::new(&format!("Exported to {}", filename)));
                }
                Err(e) => {
                    toast_overlay.add_toast(adw::Toast::new(&format!("Export failed: {}", e)));
                }
            }
        });
    }

    pub fn import_note_dialog(handle: WindowHandle) {
        let win = handle.borrow().window.clone();
        let toast_overlay = handle.borrow().toast_overlay.clone();

        let h = handle.clone();
        choose_import_file(&win, move |path| {
            match Storage::import_note_from_file(&path) {
                Ok(note) => {
                    let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("file");
                    toast_overlay.add_toast(adw::Toast::new(&format!(
                        "Imported note from {}",
                        filename
                    )));

                    let note_id = note.id.clone();
                    let mut state = h.borrow_mut();
                    state.notes.insert(0, note);
                    state.active_note_id = Some(note_id);
                    state.refresh_sidebar();
                    state.refresh_editor();
                }
                Err(e) => {
                    toast_overlay.add_toast(adw::Toast::new(&format!("Import failed: {}", e)));
                }
            }
        });
    }

    pub fn backup_all_notes_dialog(&self) {
        let notes_clone = self.notes.clone();
        let toast_overlay = self.toast_overlay.clone();

        choose_backup_directory(&self.window, move |dir| {
            match Storage::export_all_notes_to_dir(&notes_clone, &dir) {
                Ok(count) => {
                    toast_overlay.add_toast(adw::Toast::new(&format!("Backed up {} notes successfully", count)));
                }
                Err(e) => {
                    toast_overlay.add_toast(adw::Toast::new(&format!("Backup failed: {}", e)));
                }
            }
        });
    }

    pub fn refresh_sidebar(&mut self) {
        self.update_counts();
        self.refresh_tags_flowbox();
        self.refresh_sidebar_note_items();
    }

    fn update_counts(&self) {
        let total = self.notes.iter().filter(|n| !n.is_trashed).count();
        let favorites = self.notes.iter().filter(|n| !n.is_trashed && n.is_pinned).count();
        let trash = self.notes.iter().filter(|n| n.is_trashed).count();

        self.all_notes_badge.set_label(&total.to_string());
        self.favorites_badge.set_label(&favorites.to_string());
        self.trash_badge.set_label(&trash.to_string());

        let is_trash_active = self.filter_mode == FilterMode::Trash;
        self.empty_trash_btn.set_visible(is_trash_active && trash > 0);
        self.new_note_btn.set_visible(!is_trash_active);
    }

    fn refresh_tags_flowbox(&self) {
        // Collect unique tags
        let mut tags_set = std::collections::BTreeSet::new();
        for note in &self.notes {
            if !note.is_trashed {
                for tag in &note.tags {
                    tags_set.insert(tag.clone());
                }
            }
        }

        // Remove previous flowbox children
        while let Some(child) = self.tags_flowbox.first_child() {
            self.tags_flowbox.remove(&child);
        }

        if tags_set.is_empty() {
            self.tags_section_box.set_visible(false);
            return;
        }

        self.tags_section_box.set_visible(true);

        for tag in tags_set {
            let count = self.notes.iter().filter(|n| !n.is_trashed && n.tags.contains(&tag)).count();
            let btn = gtk4::Button::builder()
                .label(format!("#{} ({})", tag, count))
                .css_classes(["tag-filter-chip", "flat"])
                .build();

            if let FilterMode::Tag(ref active_tag) = self.filter_mode {
                if active_tag == &tag {
                    btn.add_css_class("suggested-action");
                }
            }

            let tag_name = tag.clone();
            btn.set_widget_name(&tag_name);
            self.tags_flowbox.append(&btn);
        }
    }

    fn refresh_sidebar_note_items(&mut self) {
        self.is_updating_ui.set(true);

        // Clear existing rows
        while let Some(row) = self.notes_listbox.first_child() {
            self.notes_listbox.remove(&row);
        }

        let visible = self.visible_notes();
        let mut row_to_select = None;

        for note in visible {
            let row = gtk4::ListBoxRow::new();
            row.add_css_class("note-list-row");
            row.set_widget_name(&note.id);

            let box_row = gtk4::Box::new(gtk4::Orientation::Vertical, 3);

            // Title & Pin Row
            let title_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
            let title_lbl = gtk4::Label::builder()
                .label(note.display_title())
                .halign(gtk4::Align::Start)
                .hexpand(true)
                .ellipsize(pango::EllipsizeMode::End)
                .css_classes(["note-item-title"])
                .build();
            title_box.append(&title_lbl);

            if note.is_pinned {
                let pin_icon = gtk4::Image::from_icon_name("starred-symbolic");
                pin_icon.set_icon_size(gtk4::IconSize::Normal);
                title_box.append(&pin_icon);
            }
            box_row.append(&title_box);

            // Excerpt Row
            let excerpt_lbl = gtk4::Label::builder()
                .label(note.excerpt())
                .halign(gtk4::Align::Start)
                .ellipsize(pango::EllipsizeMode::End)
                .css_classes(["note-item-snippet"])
                .build();
            box_row.append(&excerpt_lbl);

            // Bottom info: Date & Tags
            let meta_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
            meta_box.set_margin_top(2);

            let date_local = note.updated_at.with_timezone(&Local);
            let date_str = date_local.format("%b %d, %H:%M").to_string();
            let date_lbl = gtk4::Label::builder()
                .label(&date_str)
                .halign(gtk4::Align::Start)
                .hexpand(true)
                .css_classes(["note-item-date"])
                .build();
            meta_box.append(&date_lbl);

            for tag in note.tags.iter().take(2) {
                let tag_lbl = gtk4::Label::builder()
                    .label(format!("#{}", tag))
                    .css_classes(["tag-badge"])
                    .build();
                meta_box.append(&tag_lbl);
            }

            box_row.append(&meta_box);
            row.set_child(Some(&box_row));
            self.notes_listbox.append(&row);

            if self.active_note_id.as_ref() == Some(&note.id) {
                row_to_select = Some(row);
            }
        }

        if let Some(r) = row_to_select {
            self.notes_listbox.select_row(Some(&r));
        }

        self.is_updating_ui.set(false);
    }

    pub fn refresh_editor(&mut self) {
        self.is_updating_ui.set(true);

        if let Some(note) = self.active_note() {
            self.editor_container.set_visible(true);
            self.empty_state_page.set_visible(false);

            self.window_title.set_title(&note.display_title());
            self.title_entry.set_text(&note.title);

            let tags_str = if note.tags.is_empty() {
                String::new()
            } else {
                note.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(", ")
            };
            self.tags_entry.set_text(&tags_str);

            self.text_buffer.set_text(&note.content);
            self.pin_btn.set_active(note.is_pinned);

            let in_trash = note.is_trashed;
            self.trash_banner.set_visible(in_trash);
            self.trash_btn.set_visible(!in_trash);
            self.restore_btn.set_visible(in_trash);
            self.delete_perm_btn.set_visible(in_trash);

            self.title_entry.set_editable(!in_trash);
            self.tags_entry.set_editable(!in_trash);
            self.text_view.set_editable(!in_trash);
            self.format_toolbar.set_sensitive(!in_trash);

            self.update_stats();
            self.update_preview_content();
        } else {
            self.editor_container.set_visible(false);
            self.empty_state_page.set_visible(true);
            self.window_title.set_title("MyNote");
            self.trash_btn.set_visible(false);
            self.restore_btn.set_visible(false);
            self.delete_perm_btn.set_visible(false);
        }

        self.is_updating_ui.set(false);
    }

    fn update_stats(&self) {
        if let Some(note) = self.active_note() {
            let stats = format!(
                "{} words | {} characters | ~{} min read",
                note.word_count(),
                note.char_count(),
                note.reading_time_mins()
            );
            self.stats_label.set_label(&stats);
        } else {
            self.stats_label.set_label("");
        }
    }

    fn update_preview_content(&self) {
        if let Some(note) = self.active_note() {
            let pango_markup = markdown_to_pango(&note.content);
            self.preview_label.set_markup(&pango_markup);
        }
    }
}
