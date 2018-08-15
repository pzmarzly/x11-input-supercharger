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

pub fn lmb(down: bool) {
    mouse(down, "1");
}

pub fn rmb(down: bool) {
    mouse(down, "3");
}

pub fn scroll_up() {
    scroll("4");
}

pub fn scroll_down() {
    scroll("5");
}

fn scroll(direction: &str) {
    Command::new("xdotool")
        .arg("click")
        .arg(direction)
        .status()
        .unwrap();
}

fn mouse(down: bool, button: &str) {
    let kind = if down { "mousedown" } else { "mouseup" };
    Command::new("xdotool")
        .arg(kind)
        .arg(button)
        .status()
        .unwrap();
}
