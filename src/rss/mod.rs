pub mod storage;

use libadwaita as adw;
use adw::prelude::*;
use gtk4 as gtk;
use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use chrono::{Utc, Duration};

/*
 * The RSS Reader component (Senior Dev Persistent Reliable Edition).
 * Fixed: 403 Forbidden and Network Errors for Reddit.
 * Strategy:
 * - Use a more stable Redlib instance (redlib.catsarch.com) as fallback.
 * - Improve HTTP headers to mimic a real Firefox browser.
 * - Implement better error reporting for debugging.
 */

pub struct RssReader {
    pub container: gtk::Box,
}

impl RssReader {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 10);
        container.set_margin_top(20);
        container.set_margin_bottom(20);
        container.set_margin_start(20);
        container.set_margin_end(20);

        // Load feeds from persistence
        let initial_feeds = storage::load_feeds();
        let feeds = Rc::new(RefCell::new(initial_feeds));

        let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let title = gtk::Label::new(Some("Your Break-Time RSS Feed"));
        title.add_css_class("title-2");
        title.set_hexpand(true);
        title.set_halign(gtk::Align::Start);
        header.append(&title);

        let add_button = gtk::Button::from_icon_name("list-add-symbolic");
        add_button.set_tooltip_text(Some("Add new RSS feed"));
        header.append(&add_button);

        container.append(&header);

        // --- Add Feed UI Section ---
        let add_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        add_box.set_visible(false);
        let entry_input = gtk::Entry::builder()
            .placeholder_text("Enter RSS URL...")
            .hexpand(true)
            .build();
        let confirm_add = gtk::Button::with_label("Add");
        confirm_add.add_css_class("suggested-action");

        add_box.append(&entry_input);
        add_box.append(&confirm_add);
        container.append(&add_box);

        // --- Manage Feeds Section ---
        let manage_label = gtk::Label::new(Some("Active Feeds (Stored in ~/.config/break-time/feeds.json)"));
        manage_label.set_halign(gtk::Align::Start);
        manage_label.set_margin_top(10);
        container.append(&manage_label);

        let feed_list_box = gtk::ListBox::new();
        feed_list_box.add_css_class("boxed-list");
        container.append(&feed_list_box);

        // --- News Items Section ---
        let news_label = gtk::Label::new(Some("Recent News (Last 48h):"));
        news_label.set_halign(gtk::Align::Start);
        news_label.set_margin_top(20);
        container.append(&news_label);

        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let news_list_box = gtk::ListBox::new();
        news_list_box.set_selection_mode(gtk::SelectionMode::None);
        news_list_box.add_css_class("boxed-list");
        scrolled.set_child(Some(&news_list_box));

        container.append(&scrolled);

        let refresh_button = gtk::Button::with_label("Refresh All Feeds");
        refresh_button.add_css_class("pill");
        refresh_button.add_css_class("suggested-action");
        container.append(&refresh_button);

        // --- UI REFRESH LOGIC (Feed List) ---
        let feeds_ui = feeds.clone();
        let feed_list_box_ui = feed_list_box.clone();
        
        // Use a type-annotated RefCell to handle the closure recursion/referencing
        let refresh_ui_fn: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));
        let refresh_ui_fn_clone = refresh_ui_fn.clone();
        
        let refresh_ui = move || {
            while let Some(child) = feed_list_box_ui.first_child() {
                feed_list_box_ui.remove(&child);
            }
            let current = feeds_ui.borrow().clone();
            for (idx, url) in current.into_iter().enumerate() {
                let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
                row.set_margin_start(10); row.set_margin_end(10);
                row.set_margin_top(5); row.set_margin_bottom(5);

                let lbl = gtk::Label::new(Some(&url));
                lbl.set_hexpand(true); lbl.set_halign(gtk::Align::Start);
                lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);
                row.append(&lbl);

                let del_btn = gtk::Button::from_icon_name("user-trash-symbolic");
                del_btn.add_css_class("flat");
                del_btn.add_css_class("destructive-action");
                
                let feeds_del = feeds_ui.clone();
                let refresh_trigger = refresh_ui_fn_clone.clone();
                del_btn.connect_clicked(move |_| {
                    if feeds_del.borrow().len() > idx {
                        feeds_del.borrow_mut().remove(idx);
                        storage::save_feeds(feeds_del.borrow().clone());
                        if let Some(f) = &*refresh_trigger.borrow() {
                            f();
                        }
                    }
                });
                row.append(&del_btn);
                feed_list_box_ui.append(&row);
            }
        };

        let refresh_ui_rc = Rc::new(refresh_ui) as Rc<dyn Fn()>;
        *refresh_ui_fn.borrow_mut() = Some(refresh_ui_rc.clone());
        refresh_ui_rc();

        // --- ADD FEED LOGIC ---
        let feeds_add = feeds.clone();
        let entry_add = entry_input.clone();
        let add_box_confirm = add_box.clone();
        let refresh_after_add = refresh_ui_rc.clone();
        confirm_add.connect_clicked(move |_| {
            let url = entry_add.text().to_string();
            if !url.is_empty() && url.starts_with("http") {
                feeds_add.borrow_mut().push(url);
                storage::save_feeds(feeds_add.borrow().clone());
                entry_add.set_text("");
                add_box_confirm.set_visible(false);
                refresh_after_add();
            }
        });

        add_button.connect_clicked(move |_| {
            add_box.set_visible(!add_box.get_visible());
        });

        // --- REFRESH NEWS LOGIC ---
        let news_list_refresh = news_list_box.clone();
        let feeds_refresh = feeds.clone();
        refresh_button.connect_clicked(move |btn| {
            let list_box = news_list_refresh.clone();
            let btn_clone = btn.clone();
            let feeds = feeds_refresh.clone();
            
            btn_clone.set_sensitive(false);
            btn_clone.set_label("Fetching news...");

            glib::MainContext::default().spawn_local(async move {
                while let Some(child) = list_box.first_child() {
                    list_box.remove(&child);
                }

                let now = Utc::now();
                let threshold = now - Duration::days(2);

                let current_feeds = feeds.borrow().clone();
                for url in current_feeds {
                    match fetch_rss_with_fallback(&url).await {
                        Ok(feed) => {
                            let mut item_count = 0;
                            for entry in feed.entries {
                                let item_date = entry.published.or(entry.updated);
                                if item_date.map(|dt| dt > threshold).unwrap_or(true) {
                                    let row = create_feed_row(
                                        entry.title.as_ref().map(|t| t.content.clone()).unwrap_or_default(),
                                        entry.links.first().map(|l| l.href.clone()).unwrap_or_default()
                                    );
                                    list_box.append(&row);
                                    item_count += 1;
                                }
                                if item_count >= 10 { break; }
                            }
                        }
                        Err(e) => {
                            let error_label = gtk::Label::new(Some(&format!("Error: {}", e)));
                            error_label.add_css_class("error");
                            list_box.append(&error_label);
                        }
                    }
                }
                btn_clone.set_sensitive(true);
                btn_clone.set_label("Refresh All Feeds");
            });
        });

        Self {
            container,
        }
    }
}

async fn fetch_rss_with_fallback(url: &str) -> Result<feed_rs::model::Feed, String> {
    // Senior Dev Reliability Strategy:
    // Reddit is extremely hostile to non-browser requests.
    // Fallback instance 'redlib.catsarch.com' is currently very stable.
    match fetch_rss(url).await {
        Ok(feed) => Ok(feed),
        Err(e) => {
            if url.contains("reddit.com") {
                println!("Primary fetch failed: {}. Trying stable Redlib fallback...", e);
                let fallback_instance = "redlib.catsarch.com";
                let redlib_url = url.replace("www.reddit.com", fallback_instance)
                                   .replace("old.reddit.com", fallback_instance)
                                   .replace("reddit.com", fallback_instance);
                fetch_rss(&redlib_url).await.map_err(|err| format!("Reddit Error: {}. (Fallback also failed: {})", e, err))
            } else {
                Err(e)
            }
        }
    }
}

async fn fetch_rss(url: &str) -> Result<feed_rs::model::Feed, String> {
    // Mimic a very modern, standard Firefox browser on Linux to blend in
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:125.0) Gecko/20100101 Firefox/125.0")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Client Error: {}", e))?;

    let response = client.get(url)
        .header("Accept", "application/rss+xml, application/atom+xml, application/xml;q=0.9, text/xml;q=0.8, */*;q=0.7")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("DNT", "1") // Do Not Track
        .header("Upgrade-Insecure-Requests", "1")
        .send().await.map_err(|e| {
            if e.is_connect() {
                format!("Connection Refused (Server might be down)")
            } else if e.is_timeout() {
                format!("Request Timed Out")
            } else {
                format!("Network Error: {}", e)
            }
        })?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP Status: {}", response.status()));
    }

    let bytes = response.bytes().await.map_err(|e| format!("Data Error: {}", e))?;
    
    if bytes.is_empty() {
        return Err("Empty response".to_string());
    }

    feed_rs::parser::parse(&bytes[..]).map_err(|e| format!("Parse Error: {}. (Content might not be valid RSS)", e))
}

fn create_feed_row(title: String, link: String) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.set_margin_start(10); row.set_margin_end(10);
    row.set_margin_top(5); row.set_margin_bottom(5);

    let title_label = gtk::Label::new(Some(&title));
    title_label.set_hexpand(true);
    title_label.set_halign(gtk::Align::Start);
    title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    row.append(&title_label);

    let read_button = gtk::Button::with_label("Read");
    read_button.add_css_class("flat");
    read_button.connect_clicked(move |_| {
        let _ = gtk::gio::AppInfo::launch_default_for_uri(&link, None::<&gtk::gio::AppLaunchContext>);
    });

    row.append(&read_button);
    row
}
