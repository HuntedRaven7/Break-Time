use libadwaita as adw;
use adw::prelude::*;
use gtk4 as gtk;
use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use notify_rust::Notification;

/*
 * The Pomodoro Timer Component (Senior Dev Notification Edition).
 * Added: Native Linux desktop notifications via libnotify.
 */

pub struct PomodoroTimer {
    pub container: gtk::Box,
    time_label: gtk::Label,
    is_running: Rc<RefCell<bool>>,
    remaining_seconds: Rc<RefCell<u32>>,
    on_complete: Rc<dyn Fn()>,
}

impl PomodoroTimer {
    pub fn new(on_complete: impl Fn() + 'static) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 20);
        container.set_valign(gtk::Align::Center);
        container.set_halign(gtk::Align::Center);

        // Timer Display
        let time_label = gtk::Label::new(Some("25:00"));
        time_label.add_css_class("title-1");
        time_label.set_margin_bottom(20);
        container.append(&time_label);

        // Core Controls
        let controls = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        controls.set_halign(gtk::Align::Center);
        
        let start_button = gtk::Button::with_label("Start 25m");
        start_button.add_css_class("suggested-action");
        
        let long_start_button = gtk::Button::with_label("Start 50m");
        long_start_button.add_css_class("suggested-action");

        let pause_button = gtk::Button::with_label("Pause");
        pause_button.set_sensitive(false);

        controls.append(&start_button);
        controls.append(&long_start_button);
        controls.append(&pause_button);
        container.append(&controls);

        // Manual Unlock Option
        let unlock_button = gtk::Button::with_label("Manual Unlock RSS Reader");
        unlock_button.add_css_class("flat");
        unlock_button.set_margin_top(40);
        container.append(&unlock_button);

        let is_running = Rc::new(RefCell::new(false));
        let remaining_seconds = Rc::new(RefCell::new(1500));
        let on_complete = Rc::new(on_complete);

        // Main Timer Loop
        let time_label_clone = time_label.clone();
        let is_running_clone = is_running.clone();
        let remaining_seconds_clone = remaining_seconds.clone();
        let on_complete_clone = on_complete.clone();
        let pause_button_clone = pause_button.clone();

        glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
            if *is_running_clone.borrow() {
                let mut seconds = remaining_seconds_clone.borrow_mut();
                if *seconds > 0 {
                    *seconds -= 1;
                    let mins = *seconds / 60;
                    let secs = *seconds % 60;
                    time_label_clone.set_text(&format!("{:02}:{:02}", mins, secs));
                } else {
                    // Time's up!
                    *is_running_clone.borrow_mut() = false;
                    pause_button_clone.set_label("Pause");
                    pause_button_clone.set_sensitive(false);
                    
                    // --- NATIVE NOTIFICATION ---
                    let _ = Notification::new()
                        .summary("Break-Time")
                        .body("Time's up! Your RSS feed is now unlocked. Take a break!")
                        .icon("alarm-clock")
                        .show();
                    
                    on_complete_clone();
                }
            }
            glib::ControlFlow::Continue
        });

        // Event Handlers
        let is_running_start = is_running.clone();
        let remaining_seconds_start = remaining_seconds.clone();
        let pause_button_start = pause_button.clone();
        start_button.connect_clicked(move |_| {
            *is_running_start.borrow_mut() = true;
            *remaining_seconds_start.borrow_mut() = 1500;
            pause_button_start.set_label("Pause");
            pause_button_start.set_sensitive(true);
        });

        let is_running_long = is_running.clone();
        let remaining_seconds_long = remaining_seconds.clone();
        let pause_button_long = pause_button.clone();
        long_start_button.connect_clicked(move |_| {
            *is_running_long.borrow_mut() = true;
            *remaining_seconds_long.borrow_mut() = 3000;
            pause_button_long.set_label("Pause");
            pause_button_long.set_sensitive(true);
        });

        let is_running_pause = is_running.clone();
        let pause_button_action = pause_button.clone();
        pause_button.connect_clicked(move |_| {
            let mut running = is_running_pause.borrow_mut();
            *running = !*running;
            if *running {
                pause_button_action.set_label("Pause");
            } else {
                pause_button_action.set_label("Resume");
            }
        });

        let on_complete_unlock = on_complete.clone();
        unlock_button.connect_clicked(move |_| {
            on_complete_unlock();
        });

        Self {
            container,
            time_label,
            is_running,
            remaining_seconds,
            on_complete,
        }
    }
}
