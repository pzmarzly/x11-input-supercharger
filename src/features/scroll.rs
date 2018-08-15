use std::sync::mpsc::{channel, Sender};
use std::thread::{sleep, spawn};

use xdotool;
use MOMENT;

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ScrollConfig {
    pub hold: bool,
    pub speed: u32,
    pub button_id: u8,
}

#[derive(Debug)]
enum ParserModes {
    Main,
    RawButtonPress,
    RawButtonRelease,
}

type ScrollThread = Sender<()>;

#[derive(Debug)]
pub struct Scroll {
    active: Option<ScrollThread>,
    mode: ParserModes,
    speed: i64,
    release_cancels: bool,
    button_id: String,
}

use self::ParserModes::*;

impl Scroll {
    pub fn new(config: &ScrollConfig) -> Self {
        Self {
            active: None,
            mode: Main,
            speed: i64::from(config.speed),
            release_cancels: config.hold,
            button_id: config.button_id.to_string(),
        }
    }
    pub fn parse_line(&mut self, line: &str, devices: &Vec<String>) {
        match self.mode {
            Main => {
                if line.starts_with("EVENT type") {
                    if line[11..].starts_with("15") {
                        self.mode = RawButtonPress;
                    } else if self.release_cancels && line[11..].starts_with("16") {
                        self.mode = RawButtonRelease;
                    }
                }
            }
            RawButtonPress | RawButtonRelease => {
                if line.starts_with("    device:") {
                    let device = line[12..].split('(').skip(1).take(1).next().unwrap();
                    let device = &device[..device.len() - 1];
                    if !devices.iter().any(|ref s| &s[..] == device) {
                        self.mode = Main;
                    }
                } else if line.starts_with("    detail:") {
                    if line[12..] == self.button_id {
                        self.toggle();
                    }
                    self.mode = Main;
                }
            }
        }
    }
    pub fn toggle(&mut self) {
        if let Some(active) = self.active.take() {
            active.send(()).is_ok();
        } else {
            let (tx, rx) = channel();
            self.active = Some(tx);
            let speed = self.speed;
            spawn(move || {
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
            });
        }
    }
}
