pub mod storage;

use libadwaita as adw;
use adw::prelude::*;
use gtk4 as gtk;
use gtk::glib;
use std::cell::RefCell;
use std::rc::Rc;
use chrono::{Utc, Duration};

pub struct RssReader {
    pub container: gtk::Box,
}

impl RssReader {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        
        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .build();
        
        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 20);
        content_box.set_margin_top(20);
        content_box.set_margin_bottom(20);
        content_box.set_margin_start(20);
        content_box.set_margin_end(20);
        scrolled.set_child(Some(&content_box));
        container.append(&scrolled);

        // --- Header Section ---
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let title = gtk::Label::builder()
            .label("Break-Time News")
            .css_classes(vec!["title-1"])
            .halign(gtk::Align::Start)
            .hexpand(true)
            .build();
        header.append(&title);
        
        let refresh_btn = gtk::Button::builder()
            .icon_name("view-refresh-symbolic")
            .css_classes(vec!["pill", "suggested-action"])
            .build();
        header.append(&refresh_btn);
        content_box.append(&header);

        // --- Feed Management (Saved Feeds) ---
        let feed_management_group = adw::PreferencesGroup::new();
        feed_management_group.set_title("Saved Feeds");
        feed_management_group.set_description(Some("Manage your RSS and Reddit subscriptions"));
        content_box.append(&feed_management_group);

        let add_row = adw::ActionRow::new();
        add_row.set_title("Add New Feed");
        
        let add_entry = gtk::Entry::builder()
            .placeholder_text("https://reddit.com/r/rust/.rss")
            .hexpand(true)
            .valign(gtk::Align::Center)
            .build();
        add_row.add_suffix(&add_entry);
        
        let add_btn = gtk::Button::builder()
            .icon_name("list-add-symbolic")
            .css_classes(vec!["flat"])
            .valign(gtk::Align::Center)
            .build();
        add_row.add_suffix(&add_btn);
        feed_management_group.add(&add_row);

        let feeds_list = gtk::ListBox::new();
        feeds_list.add_css_class("boxed-list");
        feed_management_group.add(&feeds_list);

        // --- News Feed Section ---
        let news_group = adw::PreferencesGroup::new();
        news_group.set_title("Recent Articles");
        content_box.append(&news_group);

        let news_list = gtk::Box::new(gtk::Orientation::Vertical, 15);
        news_group.add(&news_list);

        // --- Data & Logic ---
        let feeds = Rc::new(RefCell::new(storage::load_feeds()));
        
        let feeds_list_clone = feeds_list.clone();
        let feeds_data_clone = feeds.clone();
        let update_feeds_ui = Rc::new(RefCell::new(None::<Rc<dyn Fn()>>));
        let update_feeds_ui_clone = update_feeds_ui.clone();

        let render_feeds = {
            let list = feeds_list_clone.clone();
            let data = feeds_data_clone.clone();
            let update_ui = update_feeds_ui_clone.clone();
            move || {
                while let Some(child) = list.first_child() {
                    list.remove(&child);
                }
                for (idx, url) in data.borrow().iter().enumerate() {
                    let row = adw::ActionRow::new();
                    row.set_title(url);
                    row.set_title_selectable(true);
                    
                    let del_btn = gtk::Button::builder()
                        .icon_name("user-trash-symbolic")
                        .css_classes(vec!["flat", "destructive-action"])
                        .valign(gtk::Align::Center)
                        .build();
                    
                    let data_inner = data.clone();
                    let update_inner = update_ui.clone();
                    del_btn.connect_clicked(move |_| {
                        data_inner.borrow_mut().remove(idx);
                        storage::save_feeds(data_inner.borrow().clone());
                        if let Some(f) = update_inner.borrow().as_ref() {
                            f();
                        }
                    });
                    
                    row.add_suffix(&del_btn);
                    list.append(&row);
                }
            }
        };

        let render_feeds_rc = Rc::new(render_feeds);
        *update_feeds_ui.borrow_mut() = Some(render_feeds_rc.clone());
        render_feeds_rc();

        // Add Logic
        let feeds_add = feeds.clone();
        let add_entry_clone = add_entry.clone();
        let update_add = update_feeds_ui.clone();
        add_btn.connect_clicked(move |_| {
            let url = add_entry_clone.text().to_string();
            if !url.is_empty() {
                feeds_add.borrow_mut().push(url);
                storage::save_feeds(feeds_add.borrow().clone());
                add_entry_clone.set_text("");
                if let Some(f) = update_add.borrow().as_ref() {
                    f();
                }
            }
        });

        // Refresh Logic
        let news_list_refresh = news_list.clone();
        let feeds_refresh = feeds.clone();
        refresh_btn.connect_clicked(move |btn| {
            let list = news_list_refresh.clone();
            let btn_clone = btn.clone();
            let feeds_data = feeds_refresh.clone();

            btn_clone.set_sensitive(false);
            
            glib::MainContext::default().spawn_local(async move {
                while let Some(child) = list.first_child() {
                    list.remove(&child);
                }

                let now = Utc::now();
                let threshold = now - Duration::days(2);

                for url in feeds_data.borrow().iter() {
                    if let Ok(feed) = fetch_rss_with_fallback(url).await {
                        for entry in feed.entries.into_iter().take(8) {
                            let item_date = entry.published.or(entry.updated);
                            if item_date.map(|dt| dt > threshold).unwrap_or(true) {
                                let title = entry.title.as_ref().map(|t| t.content.clone()).unwrap_or_default();
                                let link = entry.links.first().map(|l| l.href.clone()).unwrap_or_default();
                                let summary = entry.summary.as_ref().map(|s| s.content.clone()).unwrap_or_else(|| {
                                    entry.content.as_ref().and_then(|c| c.body.clone()).unwrap_or_default()
                                });

                                let card = create_feed_card(title, link, summary);
                                list.append(&card);
                            }
                        }
                    }
                }
                btn_clone.set_sensitive(true);
            });
        });

        Self { container }
    }
}

async fn fetch_rss_with_fallback(url: &str) -> Result<feed_rs::model::Feed, String> {
    match fetch_rss(url).await {
        Ok(feed) => Ok(feed),
        Err(e) => {
            if url.contains("reddit.com") {
                let fallback_instance = "redlib.catsarch.com";
                let redlib_url = url.replace("www.reddit.com", fallback_instance)
                                   .replace("old.reddit.com", fallback_instance)
                                   .replace("reddit.com", fallback_instance);
                fetch_rss(&redlib_url).await.map_err(|err| format!("Reddit Fallback Error: {}", err))
            } else {
                Err(e)
            }
        }
    }
}

async fn fetch_rss(url: &str) -> Result<feed_rs::model::Feed, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:125.0) Gecko/20100101 Firefox/125.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    feed_rs::parser::parse(&bytes[..]).map_err(|e| e.to_string())
}

fn create_feed_card(title: String, link: String, summary: String) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 8);
    card.add_css_class("card");
    card.set_margin_start(5);
    card.set_margin_end(5);
    card.set_margin_top(5);
    card.set_margin_bottom(5);
    
    // Content Box
    let content_vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content_vbox.set_margin_top(15);
    content_vbox.set_margin_bottom(15);
    content_vbox.set_margin_start(15);
    content_vbox.set_margin_end(15);
    content_vbox.set_hexpand(true);

    let title_lbl = gtk::Label::builder()
        .label(&title)
        .css_classes(vec!["bold"])
        .halign(gtk::Align::Start)
        .wrap(true)
        .xalign(0.0)
        .build();
    
    let clean_summary = strip_html_tags(&summary);
    let snippet = if clean_summary.len() > 180 {
        format!("{}...", &clean_summary[..180])
    } else {
        clean_summary
    };

    let summary_lbl = gtk::Label::builder()
        .label(&snippet)
        .css_classes(vec!["dim-label", "caption"])
        .halign(gtk::Align::Start)
        .wrap(true)
        .xalign(0.0)
        .build();

    let read_btn = gtk::Button::builder()
        .label("Read More")
        .css_classes(vec!["flat", "suggested-action"])
        .halign(gtk::Align::Start)
        .build();
    
    let l_clone = link.clone();
    read_btn.connect_clicked(move |_| {
        let _ = gtk::gio::AppInfo::launch_default_for_uri(&l_clone, None::<&gtk::gio::AppLaunchContext>);
    });

    content_vbox.append(&title_lbl);
    content_vbox.append(&summary_lbl);
    content_vbox.append(&read_btn);

    card.append(&content_vbox);

    card
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for c in input.chars() {
        if c == '<' { in_tag = true; }
        else if c == '>' { in_tag = false; }
        else if !in_tag { output.push(c); }
    }
    output.replace("&amp;", "&").replace("&quot;", "\"").replace("&apos;", "'").trim().to_string()
}

