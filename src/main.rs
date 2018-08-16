#[macro_use]
extern crate const_cstr;
#[macro_use]
extern crate lazy_panic;
extern crate ctrlc;
extern crate x11;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

mod config;
mod features;
mod util;
mod x;
mod xdotool;
mod xmodmap;

use lazy_panic::formatter;

use std::alloc::System;
use std::time::Duration;

use config::Config;
use features::keyboard_click::KeyboardClick;
use features::scroll::Scroll;
use util::need_dep;
use x::X;

// Xlib sometimes chokes and crashes with jemalloc, while calling XNextEvent
// TODO: check whether necessary anymore
#[global_allocator]
static ALLOCATOR: System = System;

const MOMENT: Duration = Duration::from_millis(4);

fn main() {
    set_panic_message!(formatter::Simple);

    need_dep("xdotool");
    need_dep("xmodmap");

    let config = Config::load();
    if config.scroll.is_none() && config.keyboard_click.is_none() {
        panic!("Current configuration does nothing - all features disabled");
    }

    let mut x = X::new();

    let mut scroll = config.scroll.as_ref().map(|c| Scroll::new(c, &mut x));
    let mut keyboard_click = config
        .keyboard_click
        .as_ref()
        .map(|c| KeyboardClick::new(c, &mut x));

    keyboard_click.as_ref().map(|c| c.register_ctrlc_handler());

    let mut x = x.finish();
    loop {
        if let Some(ev) = x.poll() {
            scroll.as_mut().map(|o| o.handle(&ev));
            keyboard_click.as_mut().map(|o| o.handle(&ev));
        }
    }
}
