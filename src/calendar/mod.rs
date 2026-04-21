use gtk4 as gtk;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use chrono::{Datelike, Local, NaiveDate};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CalendarEvent {
    pub id: String,
    pub date: String, // Format: YYYY-MM-DD
    pub title: String,
}

pub struct CalendarView {
    pub container: gtk::Box,
    events: Rc<RefCell<Vec<CalendarEvent>>>,
}

impl CalendarView {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.set_hexpand(true);
        container.set_vexpand(true);

        let events = Rc::new(RefCell::new(Self::load_events()));

        // --- LEFT SIDE: CALENDAR ---
        let cal_section = gtk::Box::new(gtk::Orientation::Vertical, 10);
        cal_section.set_margin_start(20);
        cal_section.set_margin_end(20);
        cal_section.set_margin_top(20);
        cal_section.set_margin_bottom(20);
        cal_section.set_hexpand(true);
        cal_section.set_vexpand(true);

        let calendar = gtk::Calendar::new();
        calendar.set_hexpand(true);
        calendar.set_vexpand(true);
        cal_section.append(&calendar);

        // --- RIGHT SIDE: EVENTS ---
        let event_section = gtk::Box::new(gtk::Orientation::Vertical, 15);
        event_section.set_width_request(350);
        event_section.set_margin_start(20);
        event_section.set_margin_end(20);
        event_section.set_margin_top(20);
        event_section.set_margin_bottom(20);

        let date_label = gtk::Label::builder()
            .label("Select a date")
            .css_classes(vec!["title-2"])
            .halign(gtk::Align::Start)
            .build();
        event_section.append(&date_label);

        // Event List
        let event_list_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .child(&event_list_box)
            .vexpand(true)
            .build();
        event_section.append(&scrolled);

        // Add Event UI
        let add_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let event_entry = gtk::Entry::builder()
            .placeholder_text("New event...")
            .hexpand(true)
            .build();
        let add_button = gtk::Button::builder()
            .icon_name("list-add-symbolic")
            .css_classes(vec!["suggested-action"])
            .build();
        
        add_box.append(&event_entry);
        add_box.append(&add_button);
        event_section.append(&add_box);

        container.append(&cal_section);
        container.append(&event_section);

        // Marking Logic
        let update_markers = {
            let calendar = calendar.clone();
            let events = events.clone();
            move || {
                calendar.clear_marks();
                let date = calendar.date();
                let year = date.year();
                let month = date.month();
                
                let current_events = events.borrow();
                for event in current_events.iter() {
                    if let Ok(parsed_date) = NaiveDate::parse_from_str(&event.date, "%Y-%m-%d") {
                        if parsed_date.year() == year && parsed_date.month() as i32 == month {
                            calendar.mark_day(parsed_date.day());
                        }
                    }
                }
            }
        };

        // Initial markers
        update_markers();

        // Refresh markers when month/year changes
        let update_m = update_markers.clone();
        calendar.connect_closure("notify::month", false, glib::closure_local!(move |_: gtk::Calendar, _: glib::ParamSpec| {
            update_m();
        }));

        let update_y = update_markers.clone();
        calendar.connect_closure("notify::year", false, glib::closure_local!(move |_: gtk::Calendar, _: glib::ParamSpec| {
            update_y();
        }));

        // Logic
        let rerender_fn: Rc<RefCell<Option<Rc<dyn Fn(NaiveDate)>>>> = Rc::new(RefCell::new(None));

        let events_clone = events.clone();
        let event_list_box_clone = event_list_box.clone();
        let date_label_clone = date_label.clone();
        let rerender_clone = rerender_fn.clone();
        let update_markers_del = update_markers.clone();

        *rerender_fn.borrow_mut() = Some(Rc::new(move |date: NaiveDate| {
            date_label_clone.set_text(&date.format("%B %e, %Y").to_string());
            
            // Clear list
            while let Some(child) = event_list_box_clone.first_child() {
                event_list_box_clone.remove(&child);
            }

            let date_str = date.format("%Y-%m-%d").to_string();
            let current_events = events_clone.borrow();
            let day_events: Vec<_> = current_events.iter().filter(|e| e.date == date_str).collect();

            if day_events.is_empty() {
                let empty_label = gtk::Label::new(Some("No events for this day"));
                empty_label.add_css_class("dim-label");
                empty_label.set_margin_top(20);
                event_list_box_clone.append(&empty_label);
            } else {
                for event in day_events {
                    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
                    row.add_css_class("card");
                    row.set_margin_bottom(5);
                    
                    let label = gtk::Label::new(Some(&event.title));
                    label.set_halign(gtk::Align::Start);
                    label.set_hexpand(true);
                    label.set_margin_start(10);
                    label.set_margin_end(10);
                    label.set_margin_top(10);
                    label.set_margin_bottom(10);
                    
                    let delete_btn = gtk::Button::builder()
                        .icon_name("user-trash-symbolic")
                        .css_classes(vec!["flat", "destructive-action"])
                        .build();

                    row.append(&label);
                    row.append(&delete_btn);
                    event_list_box_clone.append(&row);

                    // Delete Logic
                    let id = event.id.clone();
                    let events_del = events_clone.clone();
                    let rerender_del = rerender_clone.clone();
                    let update_del = update_markers_del.clone();
                    delete_btn.connect_clicked(move |_| {
                        events_del.borrow_mut().retain(|e| e.id != id);
                        Self::save_events(&events_del.borrow());
                        update_del();
                        if let Some(f) = rerender_del.borrow().as_ref() {
                            f(date);
                        }
                    });
                }
            }
        }));

        // Initialize with today
        let today = Local::now().naive_local().date();
        if let Some(f) = rerender_fn.borrow().as_ref() {
            f(today);
        }

        // Calendar selection signal
        let rerender_cal = rerender_fn.clone();
        calendar.connect_day_selected(move |cal| {
            let date = cal.date();
            let naive = NaiveDate::from_ymd_opt(
                date.year(),
                date.month() as u32,
                date.day_of_month() as u32,
            ).unwrap_or(today);
            if let Some(f) = rerender_cal.borrow().as_ref() {
                f(naive);
            }
        });

        // Add Event logic
        let events_add = events.clone();
        let event_entry_clone = event_entry.clone();
        let calendar_add = calendar.clone();
        let rerender_add = rerender_fn.clone();
        
        let add_action = move || {
            let title = event_entry_clone.text().to_string();
            if !title.trim().is_empty() {
                let date = calendar_add.date();
                let date_str = format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day_of_month());
                
                let new_event = CalendarEvent {
                    id: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos()
                        .to_string(),
                    date: date_str,
                    title: title.clone(),
                };

                events_add.borrow_mut().push(new_event);
                Self::save_events(&events_add.borrow());
                update_markers.clone()();
                
                let naive = NaiveDate::from_ymd_opt(
                    date.year(),
                    date.month() as u32,
                    date.day_of_month() as u32,
                ).unwrap_or(today);
                
                if let Some(f) = rerender_add.borrow().as_ref() {
                    f(naive);
                }
                
                event_entry_clone.set_text("");
            }
        };

        add_button.connect_clicked({
            let action = add_action.clone();
            move |_| action()
        });
        event_entry.connect_activate(move |_| add_action());

        Self {
            container,
            events,
        }
    }

    fn get_data_dir() -> PathBuf {
        let proj_dirs = directories::ProjectDirs::from("io.github", "HuntedRaven7", "BreakTime")
            .expect("Could not find project directories");
        let data_dir = proj_dirs.data_dir();
        if !data_dir.exists() {
            fs::create_dir_all(data_dir).unwrap_or_default();
        }
        data_dir.to_path_buf()
    }

    fn get_file_path() -> PathBuf {
        Self::get_data_dir().join("calendar_events.json")
    }

    fn load_events() -> Vec<CalendarEvent> {
        let path = Self::get_file_path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(path) {
                if let Ok(events) = serde_json::from_str(&data) {
                    return events;
                }
            }
        }
        Vec::new()
    }

    fn save_events(events: &[CalendarEvent]) {
        let path = Self::get_file_path();
        if let Ok(json) = serde_json::to_string_pretty(events) {
            let _ = fs::write(path, json);
        }
    }
}
