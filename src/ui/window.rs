use libadwaita as adw;
use libadwaita::prelude::*;
use libadwaita::subclass::prelude::*;
use gtk4 as gtk;
use gtk::glib;

/*
 * The main application window for Break-Time.
 */

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager, gtk::gio::ActionGroup, gtk::gio::ActionMap;
}

impl Window {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    pub fn header_bar(&self) -> adw::HeaderBar {
        self.imp().header_bar.clone()
    }

}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Window {
        pub stack: adw::ViewStack,
        pub header_bar: adw::HeaderBar,
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
            main_box.append(&self.header_bar);

            let view_switcher = adw::ViewSwitcher::new();
            view_switcher.set_stack(Some(&self.stack));
            view_switcher.set_policy(adw::ViewSwitcherPolicy::Wide);
            self.header_bar.set_title_widget(Some(&view_switcher));

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
