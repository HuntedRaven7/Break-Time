mod ui;
mod timer;
mod rss;
mod notes;

use libadwaita as adw;
use adw::prelude::*;
use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk4 as gtk;
use gtk::glib;
use crate::ui::window::Window;
use crate::timer::PomodoroTimer;
use crate::rss::RssReader;
use crate::notes::NoteEditor;

/*
 * Entry point for the Break-Time application.
 */

fn main() -> glib::ExitCode {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let _guard = rt.enter();

    let application = adw::Application::builder()
        .application_id("com.example.BreakTime")
        .build();

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

    window.present();
}
