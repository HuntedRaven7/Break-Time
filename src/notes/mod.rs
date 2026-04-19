use libadwaita as adw;
use adw::prelude::*;
use gtk4 as gtk;
use gtk::prelude::*;
use sourceview5::prelude::*;
use std::fs;
use gtk::gio;

/*
 * The Markdown Notes component (Senior Dev Edition).
 * Added: 
 * - Style Scheme (Theme) selection for the editor.
 * - Improved UI layout for the theme selector.
 */

pub struct NoteEditor {
    pub container: gtk::Box,
    editor: sourceview5::View,
}

impl NoteEditor {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let top_bar = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        top_bar.set_margin_top(10);
        top_bar.set_margin_bottom(10);
        top_bar.set_margin_start(10);
        top_bar.set_margin_end(10);

        let info_label = gtk::Label::new(Some("Markdown Notes"));
        info_label.set_hexpand(true);
        info_label.set_halign(gtk::Align::Start);
        top_bar.append(&info_label);

        // --- Theme Selection Section ---
        let theme_manager = sourceview5::StyleSchemeManager::new();
        // Convert the ids to a StringList for the dropdown
        let scheme_ids: Vec<String> = theme_manager.scheme_ids().iter().map(|s| s.to_string()).collect();
        let theme_list = gtk::StringList::new(&[]);
        for id in &scheme_ids {
            theme_list.append(id);
        }

        let theme_dropdown = gtk::DropDown::builder()
            .model(&theme_list)
            .build();
        
        top_bar.append(&theme_dropdown);

        let save_button = gtk::Button::with_label("Save Note");
        save_button.add_css_class("suggested-action");
        top_bar.append(&save_button);

        container.append(&top_bar);

        // Setup the SourceView and Buffer
        let buffer = sourceview5::Buffer::new(None);
        let lang_manager = sourceview5::LanguageManager::new();
        let lang = lang_manager.language("markdown");
        buffer.set_language(lang.as_ref());
        
        // Apply default theme if available
        if let Some(scheme) = theme_manager.scheme("adwaita-dark").or_else(|| theme_manager.scheme("classic")) {
            buffer.set_style_scheme(Some(&scheme));
            // Set the dropdown to the correct index if we found a match
            if let Some(index) = scheme_ids.iter().position(|id| id == &scheme.id().to_string()) {
                theme_dropdown.set_selected(index as u32);
            }
        }

        let editor = sourceview5::View::with_buffer(&buffer);
        editor.set_vexpand(true);
        editor.set_monospace(true);
        editor.set_wrap_mode(gtk::WrapMode::Word);
        editor.set_show_line_numbers(true);

        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .child(&editor)
            .build();

        container.append(&scrolled);

        // --- Theme Change Logic ---
        let buffer_for_theme = buffer.clone();
        let theme_manager_clone = theme_manager.clone();
        let scheme_ids_clone = scheme_ids.clone();
        theme_dropdown.connect_selected_notify(move |dropdown| {
            let selected_index = dropdown.selected() as usize;
            if let Some(scheme_id) = scheme_ids_clone.get(selected_index) {
                if let Some(scheme) = theme_manager_clone.scheme(scheme_id) {
                    buffer_for_theme.set_style_scheme(Some(&scheme));
                }
            }
        });

        // --- Save Action Logic ---
        let buffer_clone = buffer.clone();
        save_button.connect_clicked(move |_| {
            let file_dialog = gtk::FileDialog::new();
            file_dialog.set_title("Save Markdown Note");
            file_dialog.set_initial_name(Some("note.md"));

            let buffer_for_save = buffer_clone.clone();
            file_dialog.save(Option::<&gtk::Window>::None, Option::<&gio::Cancellable>::None, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        let start = buffer_for_save.start_iter();
                        let end = buffer_for_save.end_iter();
                        let text = buffer_for_save.text(&start, &end, false);
                        
                        if let Err(e) = fs::write(path, text.as_str()) {
                            eprintln!("Failed to save file: {}", e);
                        } else {
                            println!("Note saved successfully!");
                        }
                    }
                }
            });
        });

        Self {
            container,
            editor,
        }
    }
}
