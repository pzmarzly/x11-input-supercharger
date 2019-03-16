use movie::actor;
use serde_derive::Deserialize;

use crate::gui::GuiThread;
use crate::x::xdotool;
use crate::x::xlib::{Event, XLib};

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ScrollConfig {
    pub device: String,
    pub subdevice: u32,
    pub hold: bool,
    pub speed: u32,
    pub button_id: u8,
    pub indicator: bool,
    pub indicator_size: u16,
    pub cancel_on_keypress: bool,
}

pub struct Scroll<'a> {
    config: &'a ScrollConfig,
    source_id: u32,
    scroll_thread: Option<ScrollThread::Handle>,
    gui_thread: GuiThread::Handle,
}

impl<'a> Scroll<'a> {
    pub fn new(config: &'a ScrollConfig, x: &mut XLib) -> Self {
        use x11::xinput2::*;
        let source_id = x
            .get_device_id(&config.device, config.subdevice)
            .expect("Incorrect device configuration for scrolling feature");
        x.grab(&[XI_RawButtonPress, XI_RawButtonRelease]);
        if config.cancel_on_keypress {
            x.grab(&[XI_RawKeyPress]);
        }

        let gui_thread = GuiThread::Actor {
            crosshair_size: config.indicator_size,
        }
        .start();

        Self {
            config,
            source_id,
            scroll_thread: None,
            gui_thread,
        }
    }
    #[allow(clippy::if_same_then_else)]
    #[allow(clippy::collapsible_if)]
    pub fn handle(&mut self, ev: &Event) {
        use x11::xinput2::*;
        if ev.source_id == self.source_id && ev.detail == self.config.button_id {
            if ev.kind == XI_RawButtonPress {
                self.toggle();
            } else if self.config.hold && ev.kind == XI_RawButtonRelease {
                self.toggle();
            }
        } else if self.config.cancel_on_keypress && ev.kind == XI_RawKeyPress {
            if self.scroll_thread.is_some() {
                self.toggle();
            }
        }
    }
    pub fn toggle(&mut self) {
        if let Some(scroll_thread) = self.scroll_thread.take() {
            self.gui_thread.send(GuiThread::Input::HideCrosshair);
            scroll_thread.stop();
        } else {
            self.gui_thread.send(GuiThread::Input::ShowCrosshair);
            self.scroll_thread = Some(
                ScrollThread::Actor {
                    speed: self.config.speed as i64,
                }
                .start(),
            );
        }
    }
}

actor! {
    ScrollThread
    data:
        pub speed: i64,
    on_init:
        let original_y = super::xdotool::get_current_y();
        let mut progress_towards_next_event: i64 = 0;
    tick_interval: 16,
    on_tick:
        let current_y = super::xdotool::get_current_y();
        let diff = i64::from(current_y) - i64::from(original_y);

        if diff < 0 && progress_towards_next_event > 0 {
            progress_towards_next_event = 0;
        }
        if diff > 0 && progress_towards_next_event < 0 {
            progress_towards_next_event = 0;
        }

        // 4 below is compensating for change in interval (from 4 to 16)
        progress_towards_next_event += 4 * diff * self.speed;

        const THRESHOLD: i64 = 1_000_000_000;
        if progress_towards_next_event > THRESHOLD {
            while progress_towards_next_event > THRESHOLD {
                super::xdotool::scroll_down();
                progress_towards_next_event -= THRESHOLD;
            }
        } else if progress_towards_next_event < -THRESHOLD {
            while progress_towards_next_event < -THRESHOLD {
                super::xdotool::scroll_up();
                progress_towards_next_event += THRESHOLD;
            }
        }
}
