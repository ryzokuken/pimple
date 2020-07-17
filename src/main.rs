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

        let button = Button::with_label("Click me!");
        button.connect_clicked(|_| {
            println!("Clicked!");
        });
        window.add(&button);

        window.show_all();
    });

    application.run(&[]);
}
