#[macro_use]
extern crate lazy_panic;
extern crate ctrlc;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

mod config;
mod features;
mod util;
mod xdotool;
mod xinput;

use lazy_panic::formatter;

use std::io::Read;
use std::sync::mpsc::channel;
use std::thread::sleep;
use std::time::Duration;

use config::Config;
use features::keyboard_click::KeyboardClick;
use features::scroll::Scroll;
use util::need_dep;

const MOMENT: Duration = Duration::from_millis(4);

fn main() {
    set_panic_message!(formatter::Simple);

    need_dep("xinput");
    need_dep("xdotool");
    need_dep("xmodmap");
    need_dep("grep");
    need_dep("cut");

    let config = Config::load();
    if config.scroll.is_none() && config.keyboard_click.is_none() {
        panic!("Current configuration does nothing - all features disabled");
    }

    let devices = xinput::list(config.xinput_grep);
    if devices.len() == 0 {
        panic!("xinput_grep pattern did not match any input device");
    }

    let mut scroll = if let Some(ref cfg) = config.scroll {
        Some(Scroll::new(cfg))
    } else {
        None
    };
    let mut keyboard_click = if let Some(ref cfg) = config.keyboard_click {
        Some(KeyboardClick::new(cfg))
    } else {
        None
    };

    let kill = if let Some(ref keyboard_click) = keyboard_click {
        keyboard_click.reset_keys_on_ctrlc()
    } else {
        channel().1
    };

    let mut child = xinput::test_xi2();

    let mut buf = vec![0u8; 64 * 1024 * 1024];
    loop {
        let num = child.read(&mut buf).unwrap();
        if num == 0 {
            if kill.try_recv().is_ok() {
                break;
            }
            sleep(MOMENT);
            continue;
        }

        let lines = String::from_utf8_lossy(&buf[0..num]);
        for line in lines.split('\n') {
            if let Some(ref mut scroll) = scroll {
                scroll.parse_line(line, &devices);
            }
            if let Some(ref mut keyboard_click) = keyboard_click {
                keyboard_click.parse_line(line, &devices);
            }
        }
    }
}
