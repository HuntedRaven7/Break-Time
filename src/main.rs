mod ui;
mod timer;
mod rss;
mod notes;
mod todo;
mod calendar;
mod themes;

use libadwaita as adw;
use adw::prelude::*;
use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk4 as gtk;
use gtk::glib;
use std::rc::Rc;
use crate::ui::window::Window;
use crate::timer::PomodoroTimer;
use crate::rss::RssReader;
use crate::notes::NoteEditor;
use crate::todo::TodoList;
use crate::calendar::CalendarView;
use crate::themes::{Theme, ThemeManager};

/*
 * Entry point for the Break-Time application.
 */

fn main() -> glib::ExitCode {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let _guard = rt.enter();

    let application = adw::Application::builder()
        .application_id("io.github.HuntedRaven7.BreakTime")
        .build();

    application.connect_startup(|_| {
        let theme = ThemeManager::load_settings();
        ThemeManager::apply_theme(theme);

        // Load structural CSS for the Integrated Work Workspace
        let provider = gtk::CssProvider::new();
        provider.load_from_string("
            .work-separator {
                border-right: none;
            }
            paned > separator {
                min-width: 2px;
                background-color: alpha(currentColor, 0.3);
            }
            .note-editor-container {
                background-color: @view_bg_color;
                color: @view_fg_color;
            }
        ");
        if let Some(display) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    });

    application.connect_activate(build_ui);

    application.run()
}

fn build_ui(app: &adw::Application) {
    let window = Window::new(app);
    let imp = window.imp();

    // 1. Setup the Pomodoro Timer Section
    let window_clone = window.clone();
    let timer = PomodoroTimer::new(move || {
        window_clone.unlock_rss();
    });
    
    let timer_page = imp.stack.add_titled(&timer.container, Some("timer"), "Timer");
    timer_page.set_icon_name(Some("alarm-symbolic"));

    // 2. Setup the RSS Reader Section (With "Blurred" Overlay)
    let rss = RssReader::new();
    
    // Senior Dev Strategy: Use an Overlay to create a "Locked" visual state
    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&rss.container));

    // Create the "Locked" screen
    let lock_overlay = gtk::Box::new(gtk::Orientation::Vertical, 10);
    lock_overlay.set_valign(gtk::Align::Fill);
    lock_overlay.set_halign(gtk::Align::Fill);
    lock_overlay.add_css_class("background"); // Gives it a solid look
    lock_overlay.set_opacity(0.8); // Make it semi-transparent (blurred look)
    
    let lock_icon = gtk::Image::from_icon_name("changes-prevent-symbolic");
    lock_icon.set_pixel_size(64);
    lock_icon.set_margin_top(100);
    
    // Updated text to mention the manual option
    let lock_label = gtk::Label::new(Some("Complete a Pomodoro to Unlock your Feeds"));
    lock_label.set_markup("<span font_weight='bold' size='large'>Complete a Pomodoro to Unlock your Feeds</span>\n<span size='small' alpha='70%'>(or use the Manual Unlock button in the Timer tab)</span>");
    lock_label.set_justify(gtk::Justification::Center);
    
    lock_overlay.append(&lock_icon);
    lock_overlay.append(&lock_label);
    
    overlay.add_overlay(&lock_overlay);
    
    rss.container.set_sensitive(false);

    let rss_page = imp.stack.add_titled(&overlay, Some("rss"), "RSS Feed");
    rss_page.set_icon_name(Some("view-list-bullet-symbolic"));

    // 3. Setup the Integrated Work Section (Notes + Todo)
    let notes = Rc::new(NoteEditor::new());
    
    let notes_clone = notes.clone();
    let notes_for_get_path = notes.clone();
    let todo = TodoList::new(
        Rc::new(move |path| {
            notes_clone.open_file(std::path::PathBuf::from(path));
        }),
        Rc::new(move || {
            notes_for_get_path.get_current_path()
        })
    );

    let work_paned = gtk::Paned::new(gtk::Orientation::Horizontal);
    work_paned.set_start_child(Some(&notes.container));
    work_paned.set_end_child(Some(&todo.container));
    work_paned.set_position(400); // Default width
    work_paned.set_wide_handle(true);

    let last_position = Rc::new(std::cell::Cell::new(400));
    
    let work_paned_hide = work_paned.clone();
    let last_pos_hide = last_position.clone();
    notes.set_on_hide(move || {
        last_pos_hide.set(work_paned_hide.position());
        work_paned_hide.set_position(0);
    });

    let work_page = imp.stack.add_titled(&work_paned, Some("work"), "Todo & Notes");
    work_page.set_icon_name(Some("view-list-bullet-symbolic"));

    // Add a toggle button for the notes flap in the header bar
    let toggle_notes_btn = gtk::ToggleButton::builder()
        .icon_name("sidebar-show-symbolic")
        .tooltip_text("Toggle Notes Editor")
        .css_classes(vec!["flat"])
        .active(true)
        .build();
    
    let work_paned_clone = work_paned.clone();
    let last_pos_toggle = last_position.clone();
    toggle_notes_btn.connect_toggled(move |btn| {
        if btn.is_active() {
            let pos = last_pos_toggle.get();
            work_paned_clone.set_position(if pos > 0 { pos } else { 400 });
        } else {
            last_pos_toggle.set(work_paned_clone.position());
            work_paned_clone.set_position(0);
        }
    });

    let toggle_notes_btn_sync = toggle_notes_btn.clone();
    work_paned.connect_position_notify(move |paned| {
        let pos = paned.position();
        toggle_notes_btn_sync.set_active(pos > 0);
    });

    // Only show the toggle button when we are on the "Work" page
    let header_bar = window.header_bar();
    header_bar.pack_start(&toggle_notes_btn);
    
    let toggle_btn_visible = toggle_notes_btn.clone();
    imp.stack.connect_visible_child_name_notify(move |stack| {
        let name = stack.visible_child_name().unwrap_or_default();
        toggle_btn_visible.set_visible(name == "work");
    });
    toggle_notes_btn.set_visible(false); // Initial state

    // 5. Setup the Calendar Section (Always accessible)
    let calendar = CalendarView::new();
    let calendar_page = imp.stack.add_titled(&calendar.container, Some("calendar"), "Calendar");
    calendar_page.set_icon_name(Some("x-office-calendar-symbolic"));

    // 6. Setup the Settings Section
    let settings_page_content = adw::PreferencesPage::new();
    settings_page_content.set_title("Settings");
    settings_page_content.set_icon_name(Some("emblem-system-symbolic"));

    let theme_group = adw::PreferencesGroup::new();
    theme_group.set_title("Appearance");
    theme_group.set_description(Some("Customize the look and feel of Break-Time"));
    
    let theme_row = adw::ComboRow::new();
    theme_row.set_title("Theme");
    
    let model = gtk::StringList::new(&[]);
    for theme in Theme::all() {
        model.append(theme.name());
    }
    theme_row.set_model(Some(&model));

    let current_theme = ThemeManager::load_settings();
    let index = Theme::all().iter().position(|&t| t == current_theme).unwrap_or(0);
    theme_row.set_selected(index as u32);

    theme_row.connect_selected_notify(move |row| {
        let selected = row.selected();
        if let Some(theme) = Theme::all().get(selected as usize) {
            ThemeManager::apply_theme(*theme);
        }
    });

    theme_group.add(&theme_row);
    settings_page_content.add(&theme_group);

    let settings_page = imp.stack.add_titled(&settings_page_content, Some("settings"), "Settings");
    settings_page.set_icon_name(Some("emblem-system-symbolic"));

    window.present();
}
