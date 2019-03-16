use gtk::prelude::*;
use gtk::{init, main_iteration_do, Builder, Image, Window, WindowPosition};
use static_assets::asset_str;

use std::sync::mpsc::{channel, Sender};
use std::thread::{sleep, spawn};

use crate::MOMENT;

pub enum EventKind {
    ShowCrosshair,
    HideCrosshair,
}

pub fn gui_thread(crosshair_size: u16) -> Sender<EventKind> {
    let (ev_tx, ev_rx) = channel();
    spawn(move || {
        if init().is_err() {
            println!("Failed to initialize GTK.");
            return;
        }

        let glade_src = &asset_str!("src/gui/ui.glade");
        let builder = Builder::new_from_string(glade_src);

        let crosshair: Window = builder.get_object("crosshair").unwrap();
        let icon: Image = builder.get_object("crosshair_icon").unwrap();
        let margin = i32::from(crosshair_size);
        icon.set_margin_top(margin);
        icon.set_margin_bottom(margin);
        icon.set_margin_left(margin);
        icon.set_margin_right(margin);

        loop {
            while let Ok(ev) = ev_rx.try_recv() {
                use self::EventKind::*;
                match ev {
                    ShowCrosshair => {
                        crosshair.set_position(WindowPosition::Mouse);
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
