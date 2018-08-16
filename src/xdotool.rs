use std::process::Command;
use std::str::from_utf8_unchecked;

pub fn get_current_y() -> u16 {
    let r = Command::new("xdotool")
        .arg("getmouselocation")
        .output()
        .unwrap();
    let stdout = r.stdout;
    let stdout = unsafe { from_utf8_unchecked(&stdout) };
    let y = stdout.split(' ').skip(1).take(1).next().unwrap();
    y[2..].parse::<u16>().unwrap()
}

pub fn scroll_up() {
    scroll("4");
}

pub fn scroll_down() {
    scroll("5");
}

fn scroll(direction: &str) {
    Command::new("xdotool")
        .arg("mousedown")
        .arg(direction)
        .arg("mouseup")
        .arg(direction)
        .status()
        .unwrap();
}

pub fn click(down: bool, rmb: bool) {
    let kind = if down { "mousedown" } else { "mouseup" };
    let button = if rmb { "3" } else { "1" };
    Command::new("xdotool")
        .arg(kind)
        .arg(button)
        .status()
        .unwrap();
}

pub fn key(down: bool, num: u8) {
    let kind = if down { "keydown" } else { "keyup" };
    Command::new("xdotool")
        .arg(kind)
        .arg(num.to_string())
        .status()
        .unwrap();
}
