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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub id: String,
    pub text: String,
    pub completed: bool,
    #[serde(default)]
    pub priority: bool,
    #[serde(default)]
    pub scheduled_time: Option<String>,
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

        // Optional Time Row
        let time_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        time_row.set_halign(gtk::Align::Start);
        
        let from_label = gtk::Label::builder().label("Time (Optional):").css_classes(vec!["dim-label", "caption"]).build();
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
        add_area.append(&time_row);

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
            tasks_mut.sort_by(|a, b| b.priority.cmp(&a.priority)); // true > false
            
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

                let task = Task {
                    id: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos()
                        .to_string(),
                    text: text.clone(),
                    completed: false,
                    priority: false,
                    scheduled_time: scheduled,
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
        // Padding inside the card so it feels like a button/block
        card.set_width_request(240); // Base width so they don't look squished
        
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

        card.append(&check);
        card.append(&text_vbox);
        card.append(&priority_btn);
        card.append(&delete_btn);

        // CheckBox Toggle Logic
        let task_id = task.id.clone();
        let label_clone = label.clone();
        let tasks_clone = tasks_ref.clone();
        
        check.connect_toggled(move |c| {
            let is_completed = c.is_active();
            let mut tasks = tasks_clone.borrow_mut();
            if let Some(t) = tasks.iter_mut().find(|t| t.id == task_id) {
                t.completed = is_completed;
                let pin_prefix = if t.priority { "📌 " } else { "" };
                let escaped_text = glib::markup_escape_text(&t.text);
                
                if is_completed {
                    label_clone.set_markup(&format!("<s>{}{}</s>", pin_prefix, escaped_text));
                    label_clone.add_css_class("dim-label");
                } else {
                    label_clone.set_markup(&format!("{}{}", pin_prefix, escaped_text));
                    label_clone.remove_css_class("dim-label");
                }
            }
            Self::save_tasks(&tasks);
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
        
        let popover_pin = gtk::Button::builder()
            .label(if task.priority { "Unpin" } else { "Pin" })
            .css_classes(vec!["flat"])
            .build();
            
        let popover_del = gtk::Button::builder()
            .label("Delete")
            .css_classes(vec!["destructive-action", "flat"])
            .build();
            
        popover_box.append(&popover_pin);
        popover_box.append(&popover_del);
        popover.set_child(Some(&popover_box));
        popover.set_parent(&card);
        popover.set_has_arrow(false);

        let priority_btn_clone = priority_btn.clone();
        let popover_clone1 = popover.clone();
        popover_pin.connect_clicked(move |_| {
            priority_btn_clone.set_active(!priority_btn_clone.is_active());
            popover_clone1.popdown();
        });

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
