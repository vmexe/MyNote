use gtk4::prelude::*;
use libadwaita as adw;

pub fn show_shortcuts_dialog(parent: &adw::ApplicationWindow) {
    let builder = gtk4::Builder::new();
    let ui_str = r#"<?xml version="1.0" encoding="UTF-8"?>
<interface>
  <object class="GtkShortcutsWindow" id="shortcuts_window">
    <property name="modal">True</property>
    <child>
      <object class="GtkShortcutsSection">
        <property name="section-name">shortcuts</property>
        <property name="max-height">12</property>
        
        <child>
          <object class="GtkShortcutsGroup">
            <property name="title">General</property>
            <child>
              <object class="GtkShortcutsShortcut">
                <property name="accelerator">&lt;Primary&gt;n</property>
                <property name="title">Create new note</property>
              </object>
            </child>
            <child>
              <object class="GtkShortcutsShortcut">
                <property name="accelerator">&lt;Primary&gt;f</property>
                <property name="title">Search notes</property>
              </object>
            </child>
            <child>
              <object class="GtkShortcutsShortcut">
                <property name="accelerator">&lt;Primary&gt;s</property>
                <property name="title">Save note</property>
              </object>
            </child>
            <child>
              <object class="GtkShortcutsShortcut">
                <property name="accelerator">&lt;Primary&gt;q</property>
                <property name="title">Quit MyNote</property>
              </object>
            </child>
            <child>
              <object class="GtkShortcutsShortcut">
                <property name="accelerator">&lt;Primary&gt;question</property>
                <property name="title">Show keyboard shortcuts</property>
              </object>
            </child>
          </object>
        </child>

        <child>
          <object class="GtkShortcutsGroup">
            <property name="title">Note Actions</property>
            <child>
              <object class="GtkShortcutsShortcut">
                <property name="accelerator">&lt;Primary&gt;d</property>
                <property name="title">Toggle Favorite / Pin</property>
              </object>
            </child>
            <child>
              <object class="GtkShortcutsShortcut">
                <property name="accelerator">&lt;Primary&gt;e</property>
                <property name="title">Export note to file</property>
              </object>
            </child>
            <child>
              <object class="GtkShortcutsShortcut">
                <property name="accelerator">&lt;Primary&gt;i</property>
                <property name="title">Import note from file</property>
              </object>
            </child>
            <child>
              <object class="GtkShortcutsShortcut">
                <property name="accelerator">&lt;Primary&gt;p</property>
                <property name="title">Toggle Markdown Preview</property>
              </object>
            </child>
            <child>
              <object class="GtkShortcutsShortcut">
                <property name="accelerator">&lt;Primary&gt;Delete</property>
                <property name="title">Move note to Trash</property>
              </object>
            </child>
          </object>
        </child>

        <child>
          <object class="GtkShortcutsGroup">
            <property name="title">Markdown Formatting</property>
            <child>
              <object class="GtkShortcutsShortcut">
                <property name="accelerator">&lt;Primary&gt;b</property>
                <property name="title">Bold text (**text**)</property>
              </object>
            </child>
            <child>
              <object class="GtkShortcutsShortcut">
                <property name="accelerator">&lt;Primary&gt;k</property>
                <property name="title">Insert link [text](url)</property>
              </object>
            </child>
            <child>
              <object class="GtkShortcutsShortcut">
                <property name="accelerator">&lt;Primary&gt;&lt;Shift&gt;c</property>
                <property name="title">Insert code block</property>
              </object>
            </child>
          </object>
        </child>

      </object>
    </child>
  </object>
</interface>
"#;

    if builder.add_from_string(ui_str).is_ok() {
        if let Some(shortcuts_window) = builder.object::<gtk4::ShortcutsWindow>("shortcuts_window") {
            shortcuts_window.set_transient_for(Some(parent));
            shortcuts_window.present();
        }
    }
}
