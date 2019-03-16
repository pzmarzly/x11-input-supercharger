use movie::actor;
use serde_derive::Deserialize;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

use crate::x::xdotool;
use crate::x::xlib::{Event, XLib};
use crate::x::xmodmap;

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct KeyboardClickConfig {
    pub device: String,
    pub subdevice: u32,
    pub timeout_ms: u64,
    pub key_lmb: u8,
    pub key_rmb: u8,
    pub key_unused1: u8,
    pub key_unused2: u8,
}

#[derive(Debug, Clone)]
pub struct Keys {
    key_lmb: Vec<u64>,
    key_rmb: Vec<u64>,
    key_unused1: Vec<u64>,
    key_unused2: Vec<u64>,
}

pub struct KeyboardClick<'a> {
    timer_thread: TimerThread::Handle,
    timeout_rx: Receiver<()>,
    source_id: u32,
    config: &'a KeyboardClickConfig,
    original_keys: Keys,
    remapped: bool,
}

impl<'a> KeyboardClick<'a> {
    pub fn new(config: &'a KeyboardClickConfig, x: &mut XLib) -> Self {
        use x11::xinput2::*;
        let source_id = x
            .get_device_id(&config.device, config.subdevice)
            .expect("Incorrect device configuration for keyboard clicking feature");
        x.grab(&[XI_RawMotion, XI_RawKeyPress, XI_RawKeyRelease]);

        let original_keys = Keys {
            key_lmb: x.get_keys(config.key_lmb),
            key_rmb: x.get_keys(config.key_rmb),
            key_unused1: x.get_keys(config.key_unused1),
            key_unused2: x.get_keys(config.key_unused2),
        };

        {
            let mut transaction = xmodmap::transaction();
            transaction.bind(config.key_lmb, &[]);
            transaction.bind(config.key_rmb, &[]);
            transaction.bind(config.key_unused1, &original_keys.key_lmb);
            transaction.bind(config.key_unused2, &original_keys.key_rmb);
            transaction.commit();
        }

        let (timeout_tx, timeout_rx) = channel();
        let timeout_time = Duration::from_millis(config.timeout_ms);
        let timer_thread = TimerThread::Actor {
            timeout_tx,
            timeout_time,
        }
        .start();

        let ret = Self {
            config,
            timer_thread,
            timeout_rx,
            source_id,
            original_keys,
            remapped: false,
        };
        ret.register_ctrlc_handler();
        ret
    }
    #[allow(clippy::collapsible_if)]
    pub fn handle(&mut self, ev: &Event) {
        use x11::xinput2::*;
        if self.timeout_rx.try_recv().is_ok() {
            self.remapped = false;
        }
        if ev.source_id == self.source_id && ev.kind == XI_RawMotion {
            self.remapped = true;
            self.timer_thread.send(TimerThread::Input::Event);
        } else if ev.kind == XI_RawKeyPress || ev.kind == XI_RawKeyRelease {
            if ev.detail == self.config.key_lmb || ev.detail == self.config.key_rmb {
                if self.remapped {
                    xdotool::click(ev.kind == XI_RawKeyPress, ev.detail == self.config.key_rmb);
                } else {
                    let is_rmb = ev.detail == self.config.key_rmb;
                    let key_to_hit = if is_rmb {
                        self.config.key_unused2
                    } else {
                        self.config.key_unused1
                    };
                    xdotool::key(ev.kind == XI_RawKeyPress, key_to_hit);
                }
            }
        }
    }
    fn register_ctrlc_handler(&self) {
        let keys = self.original_keys.clone();
        let (key_lmb, key_rmb, key_unused1, key_unused2) = (
            self.config.key_lmb,
            self.config.key_rmb,
            self.config.key_unused1,
            self.config.key_unused2,
        );
        ctrlc::set_handler(move || {
            let mut transaction = xmodmap::transaction();
            transaction.bind(key_lmb, &keys.key_lmb);
            transaction.bind(key_rmb, &keys.key_rmb);
            transaction.bind(key_unused1, &keys.key_unused1);
            transaction.bind(key_unused2, &keys.key_unused2);
            transaction.commit();
            panic!("exiting...");
        })
        .unwrap();
    }
}

actor! {
    TimerThread
    input:
        Event,
    data:
        pub timeout_tx: super::Sender<()>,
        pub timeout_time: super::Duration,
    on_init:
        let mut last_event_time = super::Instant::now();
        let mut warmup = true;
        let mut remapped = false;
        let mut current_time = super::Instant::now();
    on_message:
        Event => {
            remapped = true;
            last_event_time = current_time;
        }
    tick_interval: 50,
    on_tick:
        current_time = super::Instant::now();
        let delta_time = current_time.duration_since(last_event_time);
        if warmup {
            if delta_time > self.timeout_time {
                warmup = false;
            }
        } else {
            if remapped && delta_time > self.timeout_time {
                remapped = false;
                self.timeout_tx.send(()).unwrap();
            }
        }
}
