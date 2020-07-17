extern crate gtk;
extern crate gio;

use gtk::prelude::*;
use gio::prelude::*;

fn main() {
    let application = gtk::Application::new(
        Some("dev.ryzokuken.pimple"),
        Default::default(),
    ).expect("failed to initialize gtk application");

    application.connect_activate(|app| {
        let window = gtk::ApplicationWindow::new(app);
        window.set_title("Pimple");
        window.set_default_size(350, 70);

        let b_main = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let l_main = gtk::Label::new(Some("Pimple"));
        let b_calendar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        for day in 0..7 {
            let b_day = gtk::Box::new(gtk::Orientation::Vertical, 0);
            let l_day = gtk::Label::new(Some(day.to_string().as_ref()));
            b_day.add(&l_day);
            b_calendar.pack_end(&b_day, true, false, 0);
        }
        b_main.pack_start(&l_main, false, false, 0);
        b_main.pack_end(&b_calendar, true, false, 0);
        window.add(&b_main);

        window.show_all();
    });

    application.run(&[]);
}
