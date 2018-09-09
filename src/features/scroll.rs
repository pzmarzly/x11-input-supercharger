use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{sleep, spawn};

use x::xdotool;
use x::xlib::{Event, XLib};
use MOMENT;

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ScrollConfig {
    pub device: String,
    pub subdevice: u32,
    pub hold: bool,
    pub speed: u32,
    pub button_id: u8,
}

type ScrollThread = Sender<()>;

#[derive(Debug)]
pub struct Scroll<'a> {
    config: &'a ScrollConfig,
    source_id: u32,
    active: Option<ScrollThread>,
}

impl<'a> Scroll<'a> {
    pub fn new(config: &'a ScrollConfig, x: &mut XLib) -> Self {
        use x11::xinput2::*;
        let source_id = x
            .get_device_id(&config.device, config.subdevice)
            .expect("Incorrect device configuration for scrolling feature");
        x.grab(&[XI_RawButtonPress, XI_RawButtonRelease]);

        Self {
            config,
            source_id,
            active: None,
        }
    }
    pub fn handle(&mut self, ev: &Event) {
        use x11::xinput2::*;
        if ev.source_id == self.source_id && ev.detail == self.config.button_id {
            if ev.kind == XI_RawButtonPress {
                self.toggle();
            } else if self.config.hold && ev.kind == XI_RawButtonRelease {
                self.toggle();
            }
        }
    }
    pub fn toggle(&mut self) {
        if let Some(active) = self.active.take() {
            active.send(()).is_ok();
        } else {
            let (tx, rx) = channel();
            let speed = self.config.speed;
            spawn(move || scrolling_thread(speed, rx));
            self.active = Some(tx);
        }
    }
}

fn scrolling_thread(speed: u32, rx: Receiver<()>) {
    let speed = i64::from(speed);
    let original_y = xdotool::get_current_y();
    let mut progress_towards_next_event: i64 = 0;
    loop {
        if rx.try_recv().is_ok() {
            break;
        }
        sleep(MOMENT);

        let current_y = xdotool::get_current_y();
        let diff = i64::from(current_y) - i64::from(original_y);

        if diff < 0 && progress_towards_next_event > 0 {
            progress_towards_next_event = 0;
        }
        if diff > 0 && progress_towards_next_event < 0 {
            progress_towards_next_event = 0;
        }

        progress_towards_next_event += diff * speed;

        const THRESHOLD: i64 = 1_000_000_000;
        if progress_towards_next_event > THRESHOLD {
            while progress_towards_next_event > THRESHOLD {
                xdotool::scroll_down();
                progress_towards_next_event -= THRESHOLD;
            }
        } else if progress_towards_next_event < -THRESHOLD {
            while progress_towards_next_event < -THRESHOLD {
                xdotool::scroll_up();
                progress_towards_next_event += THRESHOLD;
            }
        }
    }
}
