# X11 input supercharger

Adds system-wide:

- auto-scrolling mode - click (middle) button and drag cursor to scroll, click again to leave the mode (can be customized to use different button or to require holding the button)
- clicking using keyboard keys, active only for x ms after selected input device was last used

Can be used by mouse or graphics tablet users.

Scrolling mode requires selected button to be unbound ([example](assets/gnome-tablet-unbound.png)).

Since 0.3.0, keyboard shortcuts are not captured correctly. So if you want to e.g. bind LMB to 2, and open link in a new tab by pressing Ctrl+2 (Ctrl+LMB), unbind Ctrl+2 in your browser settings beforehand.

## Example config - Wacom Bamboo tablet

```toml
# Windows-like auto-scrolling. Press `button_id` to start scrolling,
# then move your mouse up or down. The longer the distance between the
# cursor and the starting point, the faster you scroll. Remove/comment
# out whole section if you don't want it.
[scroll]
# Device with buttons. Use `xinput list` and `xinput test-xi2 --root` to
# determine.
device = "Wacom Bamboo Pen stylus"
# Change it if you have many devices with the same name.
subdevice = 0
# `hold = false` means click once to enable, click once to disable.
# Recommended `false` on tablets, as it's annoying when you connectivity
# while using `hold = true`.
hold = false
# Scrolling speed. Has different effect on different screen resolutions.
# Recommended to set `speed` to high value and decrease system-wide
# scrolling speed as much as possible. Equation: `speed`×`distance`[px]
# ÷1_000_000_000 = emulated mouse wheel rolls.
speed = 600000
# Which button toggles scrolling. Button 2 is the middle mouse button.
# Button 3 is upper button on Wacom Bamboo Pen.
button_id = 3
# Whether to show crosshair/indicator at cursor original position
# (when the scrolling started)
indicator = true
indicator_size = 5
# Whether to stop scrolling when keyboard event occurs
cancel_on_keypress = true

# Click using keyboard. Active only until `timeout_ms` has passed since
# the last time any of grabbed devices was used. Remove/comment out
# whole section if you don't want it.
[keyboard_click]
# Device that moves or has buttons, used to determine timeout. Use
# `xinput list` and `xinput test-xi2 --root` to determine.
device = "Wacom Bamboo Pen stylus"
# Change it if you have many devices with the same name.
subdevice = 0
# How much time must pass until keys go back to normal.
timeout_ms = 500
# Key that emulates left mouse button.
# **DO NOT USE TOGGLE BUTTON (CapsLock etc.)**
key_lmb = 25 # W
# Key that emulates right mouse button.
# **DO NOT USE TOGGLE BUTTON (CapsLock etc.)**
key_rmb = 26 # E
# Key that will be used for temporary purposes.
# **DO NOT USE TOGGLE BUTTON (CapsLock etc.)**
key_unused1 = 106 # numpad /
# Key that will be used for temporary purposes.
# **DO NOT USE TOGGLE BUTTON (CapsLock etc.)**
key_unused2 = 63 # numpad *
```

## Example config - mouse

Use config above, but replace `xinput_grep` with f.e. "Gaming Mouse", and `stylus_button_id` with some button ID. Middle button ID is 2, but that requires you to unbind its pasting functionality somehow. I recommend using additional buttons if your mouse has these. Button IDs 4 and 5 are reserved for scroll events. You can check button IDs with `xinput test-xi2 --root`.

## Installation

First, install GTK 3.18+ (default in Ubuntu 16.04 and newer), `xdotool` and `xmodmap`.

Grab binary from [Releases page](https://github.com/pzmarzly/x11-input-supercharger/releases), or build it yourself by copying the source and running `cargo build --release`, or have Cargo download the sources and put binary in `PATH` for you with `cargo install x11-input-supercharger`. Rust stable toolchain is required.

`Config.toml` must be in current working directory when starting the program.

## Miscellaneous

Konsole (terminal emulator) doesn't like having a key pressed when selecting. Use Gnome Terminal.

The program grabs root X11 input device.

The code is ugly, and the program sometimes crashes on shutdown (but doesn't seem to leave the system in broken state).

KSysGuard shows the program uses 0-1% of CPU time on Intel i5 6300HQ.

If the program is unstable on your system, check out versions 0.2.x. They used text parser instead of X11 API. However, different keyboard grabbing solution was used back then, which caused lags in Chromium-based programs. See older tree for old README.

## Tip: autostart

Add following command to autostart commands:

```bash
bash -c 'cd /path/to/folder/with/config/ && /path/to/x11-input-supercharger & disown'
```

## Tip: cursor speed

```bash
xinput set-prop "Wacom Bamboo Pen stylus" "Device Accel Constant Deceleration" 1.6
```

You can run add that command to autostart commands.

## Acknowledgements

Thanks to [Bruce Byfield](https://brucebyfield.com/) for his Wacom-related articles for Linux Magazine.

Thanks to [Linux Wacom Project developers](https://linuxwacom.github.io/about/) for making it possible to use Wacom as input device on Linux.

Thanks to all X11 developers for awesome tools they created.
