use libadwaita as adw;
use gtk4 as gtk;
use gtk::gdk;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::cell::RefCell;

thread_local! {
    static PROVIDER: RefCell<Option<gtk::CssProvider>> = RefCell::new(None);
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    AdwaitaSystem,
    AdwaitaDark,
    AdwaitaLight,
    Everforest,
    Gruvbox,
    Breeze,
}

impl Theme {
    pub fn all() -> Vec<Theme> {
        vec![
            Theme::AdwaitaSystem,
            Theme::AdwaitaDark,
            Theme::AdwaitaLight,
            Theme::Everforest,
            Theme::Gruvbox,
            Theme::Breeze,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            Theme::AdwaitaSystem => "Adwaita (System)",
            Theme::AdwaitaDark => "Adwaita Dark",
            Theme::AdwaitaLight => "Adwaita Light",
            Theme::Everforest => "Everforest",
            Theme::Gruvbox => "Gruvbox",
            Theme::Breeze => "Breeze (KDE)",
        }
    }

    pub fn get_css(&self) -> &str {
        match self {
            Theme::AdwaitaSystem | Theme::AdwaitaDark | Theme::AdwaitaLight => "", // Handled by StyleManager
            Theme::Everforest => "
                window, .background { background-color: #2b3339; color: #d3c6aa; }
                headerbar { background-color: #323d43; color: #d3c6aa; }
                button.suggested-action { background-color: #a7c080; color: #2b3339; }
                button.destructive-action { background-color: #e67e80; color: #2b3339; }
                .card { background-color: #323d43; border: 1px solid #3a454a; color: #d3c6aa; }
                label { color: inherit; }
                entry { background-color: #3a454a; color: #d3c6aa; }
            ",
            Theme::Gruvbox => "
                window, .background { background-color: #282828; color: #ebdbb2; }
                headerbar { background-color: #3c3836; color: #ebdbb2; }
                button.suggested-action { background-color: #fabd2f; color: #282828; }
                button.destructive-action { background-color: #fb4934; color: #282828; }
                .card { background-color: #3c3836; border: 1px solid #504945; color: #ebdbb2; }
                label { color: inherit; }
                entry { background-color: #504945; color: #ebdbb2; }
            ",
            Theme::Breeze => "
                window, .background { background-color: #232629; color: #eff0f1; }
                headerbar { background-color: #31363b; color: #eff0f1; }
                button.suggested-action { background-color: #3daee9; color: #ffffff; }
                button.destructive-action { background-color: #ed1515; color: #ffffff; }
                .card { background-color: #31363b; border: 1px solid #4d4d4d; color: #eff0f1; }
                label { color: inherit; }
                entry { background-color: #1b1e20; color: #eff0f1; border: 1px solid #3daee9; }
            ",
        }
    }
}

pub struct ThemeManager;

impl ThemeManager {
    pub fn apply_theme(theme: Theme) {
        // 1. Handle Libadwaita Color Scheme
        let style_manager = adw::StyleManager::default();
        match theme {
            Theme::AdwaitaSystem => style_manager.set_color_scheme(adw::ColorScheme::Default),
            Theme::AdwaitaDark => style_manager.set_color_scheme(adw::ColorScheme::ForceDark),
            Theme::AdwaitaLight => style_manager.set_color_scheme(adw::ColorScheme::ForceLight),
            _ => style_manager.set_color_scheme(adw::ColorScheme::ForceDark), // Custom themes are dark-based
        }

        // 2. Handle CSS Providers
        PROVIDER.with(|p| {
            let mut p_borrow = p.borrow_mut();
            
            if let Some(old_p) = p_borrow.take() {
                if let Some(display) = gdk::Display::default() {
                    gtk::style_context_remove_provider_for_display(&display, &old_p);
                }
            }

            let css = theme.get_css();
            if !css.is_empty() {
                let new_p = gtk::CssProvider::new();
                new_p.load_from_string(css);
                
                if let Some(display) = gdk::Display::default() {
                    gtk::style_context_add_provider_for_display(
                        &display,
                        &new_p,
                        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                    );
                }
                *p_borrow = Some(new_p);
            }
        });
        
        Self::save_settings(theme);
    }

    fn get_settings_path() -> PathBuf {
        let proj_dirs = directories::ProjectDirs::from("io.github", "HuntedRaven7", "BreakTime")
            .expect("Could not find project directories");
        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).unwrap_or_default();
        }
        config_dir.join("settings.json")
    }

    pub fn load_settings() -> Theme {
        let path = Self::get_settings_path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(path) {
                if let Ok(theme) = serde_json::from_str(&data) {
                    return theme;
                }
            }
        }
        Theme::AdwaitaSystem
    }

    fn save_settings(theme: Theme) {
        let path = Self::get_settings_path();
        if let Ok(json) = serde_json::to_string(&theme) {
            let _ = fs::write(path, json);
        }
    }
}
