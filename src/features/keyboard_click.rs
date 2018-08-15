use ctrlc;

use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

use xdotool;
use MOMENT;

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct KeyboardClickConfig {
    pub warmup_ms: u64,
    pub timeout_ms: u64,
    pub key_lmb: u8,
    pub key_rmb: u8,
    pub unused_key: String,
}

#[derive(PartialEq, Debug)]
enum ParserModes {
    Main,
    KeyPress,
    KeyRelease,
    RawMotion,
}

#[derive(Debug)]
pub struct KeyboardClick {
    notify_tx: Sender<()>,
    remap_rx: Receiver<bool>,
    remapped: bool,
    mode: ParserModes,
    key_lmb: String,
    key_rmb: String,
    old_keys: Vec<u8>,
}

#[derive(Debug)]
struct KeyboardClickCore {
    last_event_time: Instant,
    remapped: bool,
    warmup: bool,
    warmup_time: Duration,
    timeout_time: Duration,
    old_keys: Vec<u8>,
    new_keys: Vec<u8>,
    remap_tx: Sender<bool>,
}

use self::ParserModes::*;

impl KeyboardClick {
    pub fn new(config: &KeyboardClickConfig) -> Self {
        let old_keys = old_keys(config.key_lmb, config.key_rmb);
        let (notify_tx, notify_rx) = channel();
        let (remap_tx, remap_rx) = channel();
        {
            let config = config.clone();
            let old_keys = old_keys.clone();
            spawn(move || {
                KeyboardClickCore::new(config, old_keys, remap_tx).run(&notify_rx);
            });
        }
        Self {
            notify_tx,
            remap_rx,
            remapped: false,
            mode: Main,
            key_lmb: config.key_lmb.to_string(),
            key_rmb: config.key_rmb.to_string(),
            old_keys,
        }
    }
    pub fn notify(&mut self) {
        self.notify_tx.send(()).unwrap();
    }
    pub fn parse_line(&mut self, line: &str, devices: &Vec<String>) {
        while let Ok(remapped) = self.remap_rx.try_recv() {
            self.remapped = remapped;
        }
        match self.mode {
            Main => {
                if line.starts_with("EVENT type") {
                    if self.remapped && line[11..].starts_with("2") {
                        self.mode = KeyPress;
                    } else if line[11..].starts_with("3") {
                        self.mode = KeyRelease;
                    } else if line[11..].starts_with("17") {
                        self.mode = RawMotion;
                    }
                }
            }
            KeyPress | KeyRelease => {
                if line.starts_with("    detail:") {
                    if line[12..] == self.key_lmb {
                        xdotool::lmb(self.mode == KeyPress);
                    } else if line[12..] == self.key_rmb {
                        xdotool::rmb(self.mode == KeyPress);
                    }
                    self.mode = Main;
                }
            }
            RawMotion => {
                if line.starts_with("    device:") {
                    let device = line[12..].split('(').skip(1).take(1).next().unwrap();
                    let device = &device[..device.len() - 1];
                    if !devices.iter().any(|ref s| &s[..] == device) {
                        self.mode = Main;
                    }
                } else if line.starts_with("    detail:") {
                    self.notify();
                }
            }
        }
    }
    pub fn reset_keys_on_ctrlc(&self) -> Receiver<()> {
        let old_keys = self.old_keys.clone();
        let (tx, rx) = channel();
        ctrlc::set_handler(move || {
            map(&old_keys);
            tx.send(()).unwrap();
        }).unwrap();
        rx
    }
}

impl KeyboardClickCore {
    fn new(config: KeyboardClickConfig, old_keys: Vec<u8>, remap_tx: Sender<bool>) -> Self {
        Self {
            last_event_time: Instant::now(),
            remapped: false,
            warmup: true,
            warmup_time: Duration::from_millis(config.warmup_ms),
            timeout_time: Duration::from_millis(config.timeout_ms),
            old_keys,
            new_keys: format!(
                "keycode {} = {}\nkeycode {} = {}",
                config.key_lmb, config.unused_key, config.key_rmb, config.unused_key
            ).into_bytes(),
            remap_tx,
        }
    }
    fn run(&mut self, rx: &Receiver<()>) {
        loop {
            sleep(MOMENT);
            let current_time = Instant::now();
            let delta_time = current_time.duration_since(self.last_event_time);
            if self.warmup {
                if delta_time > self.warmup_time {
                    if delta_time > self.timeout_time {
                        self.warmup = false;
                    }
                }
                continue;
            }
            if rx.try_recv().is_ok() {
                self.last_event_time = current_time;
            }
            if self.remapped {
                if delta_time > self.timeout_time {
                    map(&self.old_keys);
                    self.remap(false);
                }
            } else {
                if delta_time < self.timeout_time {
                    map(&self.new_keys);
                    self.remap(true);
                }
            }
        }
    }
    fn remap(&mut self, remapped: bool) {
        self.remapped = remapped;
        self.remap_tx.send(remapped).unwrap();
    }
}

fn map(keys: &[u8]) {
    let mut child = Command::new("xmodmap")
        .arg("-")
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(keys).unwrap();
    }
    child.wait().unwrap();
}

fn old_keys(key1: u8, key2: u8) -> Vec<u8> {
    let original_keys = Command::new("xmodmap")
        .arg("-pke")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .stdout
        .unwrap();
    let mut original_keys = Command::new("grep")
        .arg("-E")
        .arg(format!("keycode +{} =|keycode +{} =", key1, key2))
        .stdin(original_keys)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    while original_keys.try_wait().unwrap().is_none() {
        sleep(MOMENT);
    }
    let mut stdout = Vec::with_capacity(128);
    original_keys
        .stdout
        .unwrap()
        .read_to_end(&mut stdout)
        .unwrap();
    stdout
}
