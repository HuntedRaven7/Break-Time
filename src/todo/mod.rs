use libadwaita as adw;
use adw::prelude::*;
use gtk4 as gtk;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use gtk::glib;
use chrono::{Datelike, Local, NaiveDate, Days, Months};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Low
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Subtask {
    pub id: String,
    pub text: String,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub id: String,
    pub text: String,
    pub completed: bool,
    #[serde(default)]
    pub priority: bool, // Legacy boolean priority
    #[serde(default)]
    pub priority_level: Priority,
    #[serde(default)]
    pub scheduled_time: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub recurring: Option<String>,
    #[serde(default)]
    pub subtasks: Vec<Subtask>,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub struct TodoList {
    pub container: gtk::Box,
    flow_box: gtk::FlowBox,
    tasks: Rc<RefCell<Vec<Task>>>,
}

impl TodoList {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let top_bar = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        top_bar.set_margin_top(15);
        top_bar.set_margin_bottom(15);
        top_bar.set_margin_start(20);
        top_bar.set_margin_end(20);

        let title_label = gtk::Label::new(Some("<span font_weight='bold' size='x-large'>Tasks</span>"));
        title_label.set_use_markup(true);
        title_label.set_halign(gtk::Align::Start);
        title_label.set_hexpand(true);
        top_bar.append(&title_label);

        let clear_button = gtk::Button::builder()
            .label("Clear Completed")
            .css_classes(vec!["destructive-action"])
            .build();
        top_bar.append(&clear_button);

        container.append(&top_bar);

        // Input field for new tasks
        let add_area = gtk::Box::new(gtk::Orientation::Vertical, 8);
        add_area.set_margin_start(20);
        add_area.set_margin_end(20);
        add_area.set_margin_bottom(20);

        let main_input_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        
        let add_entry = gtk::Entry::builder()
            .placeholder_text("What needs to be done? (Press Enter to add)")
            .hexpand(true)
            .build();
            
        let add_button = gtk::Button::builder()
            .icon_name("list-add-symbolic")
            .css_classes(vec!["suggested-action"])
            .build();

        main_input_row.append(&add_entry);
        main_input_row.append(&add_button);
        add_area.append(&main_input_row);

        // Advanced Details Row (Hidden by default)
        let details_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        details_box.set_visible(false);

        let second_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        
        // Priority DropDown
        let priority_label = gtk::Label::builder().label("Priority:").css_classes(vec!["dim-label", "caption"]).build();
        let priority_list = gtk::StringList::new(&["Low", "Medium", "High"]);
        let priority_dropdown = gtk::DropDown::builder()
            .model(&priority_list)
            .selected(0)
            .build();

        // Due Date
        let due_label = gtk::Label::builder().label("Due:").css_classes(vec!["dim-label", "caption"]).build();
        let due_entry = gtk::Entry::builder()
            .placeholder_text("YYYY-MM-DD")
            .width_request(120)
            .build();

        // Recurring
        let recurring_label = gtk::Label::builder().label("Repeat:").css_classes(vec!["dim-label", "caption"]).build();
        let recurring_list = gtk::StringList::new(&["None", "Daily", "Weekly", "Monthly"]);
        let recurring_dropdown = gtk::DropDown::builder()
            .model(&recurring_list)
            .selected(0)
            .build();

        second_row.append(&priority_label);
        second_row.append(&priority_dropdown);
        second_row.append(&due_label);
        second_row.append(&due_entry);
        second_row.append(&recurring_label);
        second_row.append(&recurring_dropdown);
        
        // Optional Time Row
        let time_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        time_row.set_halign(gtk::Align::Start);
        
        let from_label = gtk::Label::builder().label("Time:").css_classes(vec!["dim-label", "caption"]).build();
        let from_entry = gtk::Entry::builder()
            .placeholder_text("From (14:00)")
            .max_length(5)
            .width_request(80)
            .build();
        let to_label = gtk::Label::builder().label("-").css_classes(vec!["dim-label"]).build();
        let to_entry = gtk::Entry::builder()
            .placeholder_text("To (15:00)")
            .max_length(5)
            .width_request(80)
            .build();

        time_row.append(&from_label);
        time_row.append(&from_entry);
        time_row.append(&to_label);
        time_row.append(&to_entry);

        details_box.append(&second_row);
        details_box.append(&time_row);
        add_area.append(&details_box);

        let toggle_details_btn = gtk::Button::builder()
            .label("Add Details")
            .css_classes(vec!["flat", "dim-label"])
            .halign(gtk::Align::Start)
            .build();
        
        let details_box_clone = details_box.clone();
        let toggle_btn_clone = toggle_details_btn.clone();
        toggle_details_btn.connect_clicked(move |_| {
            let is_visible = details_box_clone.get_visible();
            details_box_clone.set_visible(!is_visible);
            toggle_btn_clone.set_label(if is_visible { "Add Details" } else { "Hide Details" });
        });
        add_area.append(&toggle_details_btn);

        container.append(&add_area);

        // FlowBox for tasks
        let flow_box = gtk::FlowBox::new();
        flow_box.set_selection_mode(gtk::SelectionMode::None);
        flow_box.set_valign(gtk::Align::Start);
        flow_box.set_max_children_per_line(10);
        flow_box.set_min_children_per_line(1);
        flow_box.set_column_spacing(10);
        flow_box.set_row_spacing(10);
        flow_box.set_margin_start(20);
        flow_box.set_margin_end(20);
        flow_box.set_margin_bottom(20);

        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .child(&flow_box)
            .vexpand(true)
            .build();

        container.append(&scrolled);

        let tasks = Rc::new(RefCell::new(Self::load_tasks()));
        
        let rerender_fn: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));

        // Re-render logic to apply sorting whenever priority changes
        let flow_box_clone = flow_box.clone();
        let tasks_clone = tasks.clone();
        let rerender_clone = rerender_fn.clone();
        
        *rerender_fn.borrow_mut() = Some(Rc::new(move || {
            while let Some(child) = flow_box_clone.first_child() {
                flow_box_clone.remove(&child);
            }
            
            let mut tasks_mut = tasks_clone.borrow_mut();
            tasks_mut.sort_by(|a, b| {
                let p_a = &a.priority_level;
                let p_b = &b.priority_level;
                p_b.partial_cmp(p_a).unwrap_or(std::cmp::Ordering::Equal)
            });
            
            let tasks_snapshot = tasks_mut.clone();
            drop(tasks_mut); // Prevent borrow issues during element creation
            
            for task in tasks_snapshot.iter() {
                let card = Self::create_task_card(task, tasks_clone.clone(), flow_box_clone.clone(), rerender_clone.clone());
                flow_box_clone.append(&card);
            }
        }));

        let todo_list = Self {
            container,
            flow_box,
            tasks: tasks.clone(),
        };

        // Initial render
        if let Some(f) = rerender_fn.borrow().as_ref() {
            f();
        }

        // Setup add logic
        let tasks_add_clone = tasks.clone();
        let add_entry_clone = add_entry.clone();
        let rerender_add_clone = rerender_fn.clone();

        let add_action = move || {
            let text = add_entry_clone.text().to_string();
            if !text.trim().is_empty() {
                let from_t = from_entry.text().to_string();
                let to_t = to_entry.text().to_string();
                let scheduled = if !from_t.is_empty() && !to_t.is_empty() {
                    Some(format!("{} - {}", from_t, to_t))
                } else if !from_t.is_empty() {
                    Some(format!("Starts at {}", from_t))
                } else if !to_t.is_empty() {
                    Some(format!("Ends at {}", to_t))
                } else {
                    None
                };

                let priority_level = match priority_dropdown.selected() {
                    0 => Priority::Low,
                    1 => Priority::Medium,
                    2 => Priority::High,
                    _ => Priority::Low,
                };

                let due_date = due_entry.text().to_string();
                let due_date_opt = if due_date.is_empty() { None } else { Some(due_date) };

                let recurring = match recurring_dropdown.selected() {
                    1 => Some("Daily".to_string()),
                    2 => Some("Weekly".to_string()),
                    3 => Some("Monthly".to_string()),
                    _ => None,
                };

                let task = Task {
                    id: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos()
                        .to_string(),
                    text: text.clone(),
                    completed: false,
                    priority: priority_level == Priority::High, // Map High to legacy bool
                    priority_level,
                    scheduled_time: scheduled,
                    due_date: due_date_opt,
                    recurring,
                    subtasks: Vec::new(),
                    tags: Vec::new(),
                };
                tasks_add_clone.borrow_mut().push(task.clone());
                Self::save_tasks(&tasks_add_clone.borrow());
                
                if let Some(f) = rerender_add_clone.borrow().as_ref() {
                    f();
                }
                
                add_entry_clone.set_text("");
                from_entry.set_text("");
                to_entry.set_text("");
            }
        };

        add_button.connect_clicked({
            let action = add_action.clone();
            move |_| action()
        });
        
        add_entry.connect_activate(move |_| add_action());

        // Setup Clear Completed logic
        let tasks_clear_clone = tasks.clone();
        let rerender_clear_clone = rerender_fn.clone();
        clear_button.connect_clicked(move |_| {
            let mut t = tasks_clear_clone.borrow_mut();
            t.retain(|task| !task.completed);
            Self::save_tasks(&t);
            drop(t);
            if let Some(f) = rerender_clear_clone.borrow().as_ref() {
                f();
            }
        });

        todo_list
    }

    fn create_task_card(
        task: &Task, 
        tasks_ref: Rc<RefCell<Vec<Task>>>, 
        flow_box: gtk::FlowBox, 
        rerender_fn: Rc<RefCell<Option<Rc<dyn Fn()>>>>
    ) -> gtk::Box {
        let card = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        card.add_css_class("card");
        card.set_margin_top(4);
        card.set_margin_bottom(4);
        card.set_margin_start(4);
        card.set_margin_end(4);
        card.set_width_request(280); 

        // Priority Indicator (Vertical Bar)
        let priority_bar = gtk::Separator::new(gtk::Orientation::Vertical);
        priority_bar.set_width_request(4);
        let priority_class = match task.priority_level {
            Priority::High => "error",
            Priority::Medium => "warning",
            Priority::Low => "success",
        };
        priority_bar.add_css_class(priority_class);
        card.append(&priority_bar);

        let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
        main_vbox.set_hexpand(true);

        let header_hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        
        let check = gtk::CheckButton::new();
        check.set_active(task.completed);
        check.set_valign(gtk::Align::Center);
        check.set_margin_start(10);
        
        let label = gtk::Label::new(None);
        label.set_halign(gtk::Align::Start);
        label.set_valign(gtk::Align::Center);
        label.set_wrap(true);
        label.set_xalign(0.0);
        label.set_hexpand(true);
        label.set_margin_top(10);
        label.set_margin_bottom(10);
        
        let pin_prefix = if task.priority { "📌 " } else { "" };
        let escaped_text = glib::markup_escape_text(&task.text);
        
        if task.completed {
            label.set_markup(&format!("<s>{}{}</s>", pin_prefix, escaped_text));
            label.add_css_class("dim-label");
        } else {
            label.set_markup(&format!("{}{}", pin_prefix, escaped_text));
            label.remove_css_class("dim-label");
        }

        let text_vbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
        text_vbox.set_hexpand(true);
        text_vbox.set_valign(gtk::Align::Center);
        text_vbox.append(&label);

        if let Some(time) = &task.scheduled_time {
            let time_label = gtk::Label::builder()
                .label(time)
                .css_classes(vec!["caption", "dim-label"])
                .halign(gtk::Align::Start)
                .build();
            text_vbox.append(&time_label);
        }

        let priority_btn = gtk::ToggleButton::builder()
            .icon_name("starred-symbolic")
            .css_classes(vec!["flat", "circular"])
            .valign(gtk::Align::Center)
            .active(task.priority)
            .build();
            
        if task.priority {
            priority_btn.add_css_class("suggested-action");
        }

        let delete_btn = gtk::Button::builder()
            .icon_name("user-trash-symbolic")
            .css_classes(vec!["destructive-action", "flat", "circular"])
            .valign(gtk::Align::Center)
            .margin_end(10)
            .build();

        header_hbox.append(&check);
        header_hbox.append(&text_vbox);
        header_hbox.append(&priority_btn);
        header_hbox.append(&delete_btn);
        main_vbox.append(&header_hbox);

        // Due Date & Recurring Info
        if task.due_date.is_some() || task.recurring.is_some() {
            let info_hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            info_hbox.set_margin_start(45);
            info_hbox.set_margin_bottom(4);

            if let Some(due) = &task.due_date {
                let due_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
                let icon = gtk::Image::from_icon_name("calendar-events-symbolic");
                let label = gtk::Label::builder()
                    .label(due)
                    .css_classes(vec!["caption", "dim-label"])
                    .build();
                due_box.append(&icon);
                due_box.append(&label);
                info_hbox.append(&due_box);
            }

            if let Some(rec) = &task.recurring {
                let rec_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
                let icon = gtk::Image::from_icon_name("emblem-sync-symbolic");
                let label = gtk::Label::builder()
                    .label(rec)
                    .css_classes(vec!["caption", "dim-label"])
                    .build();
                rec_box.append(&icon);
                rec_box.append(&label);
                info_hbox.append(&rec_box);
            }
            main_vbox.append(&info_hbox);
        }

        // Subtasks Section
        let expander = gtk::Expander::new(Some(&format!("Subtasks ({})", task.subtasks.len())));
        expander.set_margin_start(40);
        let subtask_vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
        
        for sub in &task.subtasks {
            let sub_hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            let sub_check = gtk::CheckButton::new();
            sub_check.set_active(sub.completed);
            
            let sub_label = gtk::Label::new(Some(&sub.text));
            sub_label.set_halign(gtk::Align::Start);
            if sub.completed {
                sub_label.add_css_class("dim-label");
            }
            
            sub_hbox.append(&sub_check);
            sub_hbox.append(&sub_label);
            subtask_vbox.append(&sub_hbox);

            // Subtask completion logic
            let task_id = task.id.clone();
            let sub_id = sub.id.clone();
            let tasks_clone = tasks_ref.clone();
            let sub_label_clone = sub_label.clone();
            sub_check.connect_toggled(move |c| {
                let active = c.is_active();
                let mut tasks = tasks_clone.borrow_mut();
                if let Some(t) = tasks.iter_mut().find(|t| t.id == task_id) {
                    if let Some(s) = t.subtasks.iter_mut().find(|s| s.id == sub_id) {
                        s.completed = active;
                        if active {
                            sub_label_clone.add_css_class("dim-label");
                        } else {
                            sub_label_clone.remove_css_class("dim-label");
                        }
                    }
                }
                Self::save_tasks(&tasks);
            });
        }

        let add_sub_entry = gtk::Entry::builder()
            .placeholder_text("Add subtask...")
            .css_classes(vec!["flat"])
            .build();
        
        let task_id_sub = task.id.clone();
        let tasks_sub_clone = tasks_ref.clone();
        let rerender_sub = rerender_fn.clone();
        add_sub_entry.connect_activate(move |entry| {
            let text = entry.text().to_string();
            if !text.trim().is_empty() {
                let mut tasks = tasks_sub_clone.borrow_mut();
                if let Some(t) = tasks.iter_mut().find(|t| t.id == task_id_sub) {
                    t.subtasks.push(Subtask {
                        id: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos().to_string(),
                        text,
                        completed: false,
                    });
                }
                Self::save_tasks(&tasks);
                drop(tasks);
                if let Some(f) = rerender_sub.borrow().as_ref() {
                    f();
                }
            }
        });
        subtask_vbox.append(&add_sub_entry);

        expander.set_child(Some(&subtask_vbox));
        main_vbox.append(&expander);

        card.append(&main_vbox);

        // CheckBox Toggle Logic
        let task_id = task.id.clone();
        let label_clone = label.clone();
        let tasks_clone = tasks_ref.clone();
        
        let rerender_fn_clone = rerender_fn.clone();
        check.connect_toggled(move |c| {
            let is_completed = c.is_active();
            let mut tasks = tasks_clone.borrow_mut();
            let mut new_tasks = Vec::new();
            let mut should_rerender = false;

            if let Some(t) = tasks.iter_mut().find(|t| t.id == task_id) {
                t.completed = is_completed;
                let pin_prefix = if t.priority_level == Priority::High { "📌 " } else { "" };
                let escaped_text = glib::markup_escape_text(&t.text);
                
                if is_completed {
                    label_clone.set_markup(&format!("<s>{}{}</s>", pin_prefix, escaped_text));
                    label_clone.add_css_class("dim-label");

                    // Handle Recurring Task
                    if let Some(recurring_type) = &t.recurring {
                        let mut next_task = t.clone();
                        next_task.id = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos().to_string();
                        next_task.completed = false;
                        next_task.subtasks.iter_mut().for_each(|s| s.completed = false);

                        if let Some(due_str) = &t.due_date {
                            if let Ok(date) = NaiveDate::parse_from_str(due_str, "%Y-%m-%d") {
                                let next_date = match recurring_type.as_str() {
                                    "Daily" => date.checked_add_days(Days::new(1)),
                                    "Weekly" => date.checked_add_days(Days::new(7)),
                                    "Monthly" => date.checked_add_months(Months::new(1)),
                                    _ => Some(date),
                                };
                                if let Some(next) = next_date {
                                    next_task.due_date = Some(next.format("%Y-%m-%d").to_string());
                                }
                            }
                        }
                        new_tasks.push(next_task);
                        should_rerender = true;
                    }
                } else {
                    label_clone.set_markup(&format!("{}{}", pin_prefix, escaped_text));
                    label_clone.remove_css_class("dim-label");
                }
            }
            
            if !new_tasks.is_empty() {
                tasks.extend(new_tasks);
            }
            Self::save_tasks(&tasks);
            drop(tasks);

            if should_rerender {
                if let Some(f) = rerender_fn_clone.borrow().as_ref() {
                    f();
                }
            }
        });

        // Priority Toggle Logic
        let task_id_pri = task.id.clone();
        let tasks_pri_clone = tasks_ref.clone();
        let rerender_pri_clone = rerender_fn.clone();
        
        priority_btn.connect_toggled(move |btn| {
            let is_priority = btn.is_active();
            let mut tasks = tasks_pri_clone.borrow_mut();
            if let Some(t) = tasks.iter_mut().find(|t| t.id == task_id_pri) {
                if t.priority == is_priority {
                    return; // Prevent infinite loops if re-rendered
                }
                t.priority = is_priority;
                t.priority_level = if is_priority { Priority::High } else { Priority::Low };
            }
            Self::save_tasks(&tasks);
            drop(tasks);
            
            // Re-render to re-sort (deferred to avoid destroying the widget in its own handler)
            if let Some(f) = rerender_pri_clone.borrow().as_ref() {
                let f_clone = f.clone();
                glib::idle_add_local_once(move || f_clone());
            }
        });

        // Delete Logic
        let task_id_del = task.id.clone();
        let flow_box_clone = flow_box.clone();
        let card_clone = card.clone();
        let tasks_clone2 = tasks_ref.clone();
        
        delete_btn.connect_clicked(move |_| {
            let mut tasks = tasks_clone2.borrow_mut();
            tasks.retain(|t| t.id != task_id_del);
            Self::save_tasks(&tasks);
            drop(tasks);
            
            // Defer widget removal
            let flow_box_defer = flow_box_clone.clone();
            let card_defer = card_clone.clone();
            glib::idle_add_local_once(move || {
                flow_box_defer.remove(&card_defer);
            });
        });

        // Right Click Context Menu (Popover)
        let popover = gtk::Popover::new();
        let popover_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        
        let popover_high = gtk::Button::builder()
            .label("🔴 High Priority")
            .css_classes(vec!["flat"])
            .build();
        let popover_med = gtk::Button::builder()
            .label("🟡 Medium Priority")
            .css_classes(vec!["flat"])
            .build();
        let popover_low = gtk::Button::builder()
            .label("🟢 Low Priority")
            .css_classes(vec!["flat"])
            .build();
            
        let popover_del = gtk::Button::builder()
            .label("🗑 Delete")
            .css_classes(vec!["destructive-action", "flat"])
            .build();
            
        popover_box.append(&popover_high);
        popover_box.append(&popover_med);
        popover_box.append(&popover_low);
        popover_box.append(&popover_del);
        popover.set_child(Some(&popover_box));
        popover.set_parent(&card);
        popover.set_has_arrow(false);

        let task_id_pop = task.id.clone();
        let tasks_pop_clone = tasks_ref.clone();
        let rerender_pop = rerender_fn.clone();
        let pop_down = popover.clone();

        let set_priority = move |level: Priority| {
            let mut tasks = tasks_pop_clone.borrow_mut();
            if let Some(t) = tasks.iter_mut().find(|t| t.id == task_id_pop) {
                t.priority_level = level.clone();
                t.priority = level == Priority::High;
            }
            Self::save_tasks(&tasks);
            drop(tasks);
            pop_down.popdown();
            if let Some(f) = rerender_pop.borrow().as_ref() {
                f();
            }
        };

        let set_h = set_priority.clone();
        popover_high.connect_clicked(move |_| set_h(Priority::High));
        let set_m = set_priority.clone();
        popover_med.connect_clicked(move |_| set_m(Priority::Medium));
        let set_l = set_priority.clone();
        popover_low.connect_clicked(move |_| set_l(Priority::Low));

        let delete_btn_clone = delete_btn.clone();
        popover_del.connect_clicked(move |_| {
            delete_btn_clone.emit_clicked();
        });

        let gesture = gtk::GestureClick::new();
        gesture.set_button(3); // Right click
        let popover_clone2 = popover.clone();
        gesture.connect_pressed(move |g, n_press, x, y| {
            if n_press == 1 {
                let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
                popover_clone2.set_pointing_to(Some(&rect));
                popover_clone2.popup();
                g.set_state(gtk::EventSequenceState::Claimed);
            }
        });
        card.add_controller(gesture);

        card
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
        Self::get_data_dir().join("todos.json")
    }

    fn load_tasks() -> Vec<Task> {
        let path = Self::get_file_path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(path) {
                if let Ok(tasks) = serde_json::from_str(&data) {
                    return tasks;
                }
            }
        }
        Vec::new()
    }

    fn save_tasks(tasks: &[Task]) {
        let path = Self::get_file_path();
        if let Ok(json) = serde_json::to_string_pretty(tasks) {
            let _ = fs::write(path, json);
        }
    }
}
