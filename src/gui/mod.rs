use cairo;
use gdk::ScreenExt;
use gtk;
use gtk::prelude::*;
use gtk::{init, main_iteration_do, Builder, Window};

use std::sync::mpsc::{channel, Sender};
use std::thread::{sleep, spawn};

use MOMENT;

pub enum EventKind {
    ShowCrosshair,
    HideCrosshair,
}

pub fn gui_thread() -> Sender<EventKind> {
    let (ev_tx, ev_rx) = channel();
    spawn(move || {
        if init().is_err() {
            println!("Failed to initialize GTK.");
            return;
        }

        let glade_src = &asset_str!("src/gui/ui.glade");
        let builder = Builder::new_from_string(glade_src);

        let crosshair: Window = builder.get_object("crosshair").unwrap();

        loop {
            while let Ok(ev) = ev_rx.try_recv() {
                use self::EventKind::*;
                match ev {
                    ShowCrosshair => {
                        crosshair.set_position(gtk::WindowPosition::Mouse);
                        crosshair.show_all();
                    }
                    HideCrosshair => crosshair.hide(),
                }
            }
            main_iteration_do(false);
            sleep(MOMENT);
        }
    });
    ev_tx
}
