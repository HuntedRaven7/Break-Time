use libadwaita as adw;
use adw::prelude::*;
use gtk4 as gtk;
use gtk::glib;
use std::cell::RefCell;
use std::rc::Rc;
use notify_rust::{Notification, Timeout};

/*
 * The Pomodoro Timer Component (Senior Dev Style Edition).
 * Updated: Made the tomato emoji ABSOLUTELY GIGANTIC.
 */

#[derive(Clone, Copy)]
enum ActiveTimerKind {
    Pomodoro25,
    Pomodoro50,
    Custom,
}

fn show_timer_finished_notifications(kind: ActiveTimerKind, toast_overlay: &adw::ToastOverlay) {
    let (toast_title, desktop_summary, desktop_body) = match kind {
        ActiveTimerKind::Pomodoro25 => (
            "25 minutes are up",
            "Break-Time — session finished",
            "Your 25-minute Pomodoro is complete. Time for a break.",
        ),
        ActiveTimerKind::Pomodoro50 => (
            "50 minutes are up",
            "Break-Time — session finished",
            "Your 50-minute session is complete. Time for a break.",
        ),
        ActiveTimerKind::Custom => (
            "Timer finished",
            "Break-Time — timer finished",
            "Your custom timer has reached zero.",
        ),
    };

    let toast = adw::Toast::new(toast_title);
    toast.set_timeout(6);
    toast_overlay.add_toast(toast);

    if let Err(e) = Notification::new()
        .summary(desktop_summary)
        .body(desktop_body)
        .appname("Break-Time")
        .icon("alarm-symbolic")
        .timeout(Timeout::Milliseconds(12_000))
        .show()
    {
        eprintln!("Break-Time: could not show desktop notification: {e}");
    }
}

pub struct PomodoroTimer {
    pub container: adw::ToastOverlay,
    _time_label: gtk::Label,
    _is_running: Rc<RefCell<bool>>,
    _remaining_seconds: Rc<RefCell<u32>>,
}

impl PomodoroTimer {
    pub fn new() -> Self {
        let toast_overlay = adw::ToastOverlay::new();

        let content = gtk::Box::new(gtk::Orientation::Vertical, 20);
        content.set_valign(gtk::Align::Center);
        content.set_halign(gtk::Align::Center);
        toast_overlay.set_child(Some(&content));

        // --- BIG TOMATO EMOJI ---
        let tomato_label = gtk::Label::new(Some("🍅"));
        tomato_label.set_margin_bottom(10);
        // Using exactly 120pt size
        tomato_label.set_markup("<span size='122880'>🍅</span>");
        content.append(&tomato_label);

        // Timer Display
        let time_label = gtk::Label::new(Some("25:00"));
        time_label.add_css_class("title-1");
        time_label.set_margin_bottom(20);
        content.append(&time_label);

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
        content.append(&controls);

        // Custom Timer Controls
        let custom_controls = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        custom_controls.set_halign(gtk::Align::Center);
        custom_controls.set_margin_top(10);

        let hours_spin = gtk::SpinButton::with_range(0.0, 99.0, 1.0);
        hours_spin.set_value(0.0);
        let hours_label = gtk::Label::new(Some("h"));

        let mins_spin = gtk::SpinButton::with_range(0.0, 59.0, 1.0);
        mins_spin.set_value(10.0);
        let mins_label = gtk::Label::new(Some("m"));

        let secs_spin = gtk::SpinButton::with_range(0.0, 59.0, 1.0);
        secs_spin.set_value(0.0);
        let secs_label = gtk::Label::new(Some("s"));

        let custom_start = gtk::Button::with_label("Start Custom");
        custom_start.add_css_class("suggested-action");
        custom_start.set_margin_start(10);

        custom_controls.append(&hours_spin);
        custom_controls.append(&hours_label);
        custom_controls.append(&mins_spin);
        custom_controls.append(&mins_label);
        custom_controls.append(&secs_spin);
        custom_controls.append(&secs_label);
        custom_controls.append(&custom_start);
        content.append(&custom_controls);

        let is_running = Rc::new(RefCell::new(false));
        let remaining_seconds = Rc::new(RefCell::new(1500));
        let active_kind = Rc::new(RefCell::new(ActiveTimerKind::Pomodoro25));

        // Main Timer Loop
        let time_label_clone = time_label.clone();
        let is_running_clone = is_running.clone();
        let remaining_seconds_clone = remaining_seconds.clone();
        let pause_button_clone = pause_button.clone();
        let active_kind_clone = active_kind.clone();
        let toast_for_tick = toast_overlay.clone();

        glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
            if *is_running_clone.borrow() {
                let mut seconds = remaining_seconds_clone.borrow_mut();
                if *seconds > 0 {
                    *seconds -= 1;
                    let hrs = *seconds / 3600;
                    let mins = (*seconds % 3600) / 60;
                    let secs = *seconds % 60;
                    if hrs > 0 {
                        time_label_clone.set_text(&format!("{:02}:{:02}:{:02}", hrs, mins, secs));
                    } else {
                        time_label_clone.set_text(&format!("{:02}:{:02}", mins, secs));
                    }
                } else {
                    // Time's up!
                    *is_running_clone.borrow_mut() = false;
                    pause_button_clone.set_label("Pause");
                    pause_button_clone.set_sensitive(false);

                    let kind = *active_kind_clone.borrow();
                    show_timer_finished_notifications(kind, &toast_for_tick);
                }
            }
            glib::ControlFlow::Continue
        });

        // Event Handlers
        let is_running_start = is_running.clone();
        let remaining_seconds_start = remaining_seconds.clone();
        let pause_button_start = pause_button.clone();
        let active_kind_start = active_kind.clone();
        start_button.connect_clicked(move |_| {
            *active_kind_start.borrow_mut() = ActiveTimerKind::Pomodoro25;
            *is_running_start.borrow_mut() = true;
            *remaining_seconds_start.borrow_mut() = 1500;
            pause_button_start.set_label("Pause");
            pause_button_start.set_sensitive(true);
        });

        let is_running_long = is_running.clone();
        let remaining_seconds_long = remaining_seconds.clone();
        let pause_button_long = pause_button.clone();
        let active_kind_long = active_kind.clone();
        long_start_button.connect_clicked(move |_| {
            *active_kind_long.borrow_mut() = ActiveTimerKind::Pomodoro50;
            *is_running_long.borrow_mut() = true;
            *remaining_seconds_long.borrow_mut() = 3000;
            pause_button_long.set_label("Pause");
            pause_button_long.set_sensitive(true);
        });

        let is_running_custom = is_running.clone();
        let remaining_seconds_custom = remaining_seconds.clone();
        let pause_button_custom = pause_button.clone();
        let hours_spin_clone = hours_spin.clone();
        let mins_spin_clone = mins_spin.clone();
        let secs_spin_clone = secs_spin.clone();
        let active_kind_custom = active_kind.clone();
        custom_start.connect_clicked(move |_| {
            let h = hours_spin_clone.value() as u32;
            let m = mins_spin_clone.value() as u32;
            let s = secs_spin_clone.value() as u32;
            let total = (h * 3600) + (m * 60) + s;

            if total > 0 {
                *active_kind_custom.borrow_mut() = ActiveTimerKind::Custom;
                *is_running_custom.borrow_mut() = true;
                *remaining_seconds_custom.borrow_mut() = total;
                pause_button_custom.set_label("Pause");
                pause_button_custom.set_sensitive(true);
            }
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


        Self {
            container: toast_overlay,
            _time_label: time_label,
            _is_running: is_running,
            _remaining_seconds: remaining_seconds,
        }
    }
}
