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
 * Senior Dev Note: We initialize a Tokio runtime here and 'enter' its context.
 * This ensures that any async libraries (like reqwest) can find the Tokio reactor,
 * even while we are running the GTK main loop.
 */

fn main() -> glib::ExitCode {
    // Initialize Tokio runtime
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let _guard = rt.enter(); // This handles the "no reactor running" panic

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

    // 2. Setup the RSS Reader Section (Locked initially)
    let rss = RssReader::new();
    let rss_page = imp.stack.add_titled(&rss.container, Some("rss"), "RSS Feed");
    rss_page.set_icon_name(Some("view-list-bullet-symbolic"));
    rss_page.set_visible(false); // Initially locked!

    // 3. Setup the Markdown Note Taking Section (Always accessible)
    let notes = NoteEditor::new();
    let notes_page = imp.stack.add_titled(&notes.container, Some("notes"), "Notes");
    notes_page.set_icon_name(Some("document-edit-symbolic"));

    window.present();
}
