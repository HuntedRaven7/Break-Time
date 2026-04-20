use libadwaita as adw;
use libadwaita::prelude::*;
use libadwaita::subclass::prelude::*;
use gtk4 as gtk;
use gtk::glib;

/* 
 * The main application window for Break-Time.
 * Updated: Senior Dev Robust Unlocking Logic.
 */

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    // This method is called to unlock the RSS section
    pub fn unlock_rss(&self) {
        let imp = self.imp();
        
        // Idempotency: don't run unlocking logic twice
        if imp.rss_unlocked.get() {
            return;
        }
        imp.rss_unlocked.set(true);
        
        // Find the RSS page (the Overlay)
        if let Some(rss_child) = imp.stack.child_by_name("rss") {
            // 1. Re-enable the entire tab content
            rss_child.set_sensitive(true);
            
            // 2. Dig into the overlay to find and enable the RSS container specifically
            if let Some(overlay) = rss_child.downcast_ref::<gtk::Overlay>() {
                if let Some(rss_container) = overlay.child() {
                    rss_container.set_sensitive(true);
                }
                
                // 3. Remove the lock screen definitively
                // Overlays added via add_overlay are siblings of the main child
                let mut current = overlay.first_child();
                while let Some(child) = current {
                    let next = child.next_sibling();
                    // We check if it's the lock box (which we added second)
                    if child != overlay.child().unwrap() {
                        overlay.remove_overlay(&child);
                    }
                    current = next;
                }
            }
        }
        println!("RSS Reader is now fully unlocked and active!");
    }
}

mod imp {
    use std::cell::Cell;
    use super::*;

    #[derive(Debug, Default)]
    pub struct Window {
        pub rss_unlocked: Cell<bool>,
        pub stack: adw::ViewStack,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "BreakTimeWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();
            
            let obj = self.obj();
            obj.set_title(Some("Break-Time"));
            obj.set_default_size(800, 600);

            let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            let header_bar = adw::HeaderBar::new();
            main_box.append(&header_bar);

            let view_switcher = adw::ViewSwitcher::new();
            view_switcher.set_stack(Some(&self.stack));
            view_switcher.set_policy(adw::ViewSwitcherPolicy::Wide);
            header_bar.set_title_widget(Some(&view_switcher));

            self.stack.set_vexpand(true);
            main_box.append(&self.stack);

            obj.set_content(Some(&main_box));
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}
