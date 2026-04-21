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

    // 3. Setup the Markdown Note Taking Section (Always accessible)
    let notes = NoteEditor::new();
    let notes_page = imp.stack.add_titled(&notes.container, Some("notes"), "Notes");
    notes_page.set_icon_name(Some("document-edit-symbolic"));

    // 4. Setup the Todo List Section (Always accessible)
    let todo = TodoList::new();
    let todo_page = imp.stack.add_titled(&todo.container, Some("todo"), "Todo");
    todo_page.set_icon_name(Some("task-due-symbolic"));

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
