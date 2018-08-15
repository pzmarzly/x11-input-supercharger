# X11 input supercharger

Adds system-wide:

- scrolling mode - click (middle) button and drag cursor to scroll, click again to leave the mode (can be customized to use different button or to require holding the button)
- clicking using keyboard keys, active only for x ms after selected input device was last used

Can be used by mouse or graphics tablet users.

Scrolling mode requires selected button to be unbound ([example](assets/gnome-tablet-unbound.png)).

## Example config - Wacom Bamboo tablet

```toml
# Use `xinput list` to get device list. Wacom Bamboo registers 4
# devices. The relevant one is called "Wacom Bamboo Pen stylus". But
# there is nothing wrong with grabbing all 4 of them, which is why I use
# "Wacom" filter to catch them all. `_grep` part comes from the fact
# that the program is just running `xinput list | grep ${xinput_grep}`.
xinput_grep = "Wacom"

# Windows-like auto-scrolling. Press `button_id` to start scrolling,
# then move your mouse up or down. The longer the distance between the
# cursor and the starting point, the faster you scroll. Remove/comment
# out whole section if you don't want it.
[scroll]
# `hold = false` means click once to enable, click once to disable.
# Recommended `false` on tablets, as it's annoying when you connectivity
# while using `hold = true`.
hold = false
# Scrolling speed. Has different effect on different screen resolutions.
# Recommended to set `speed` to high value and decrease system-wide
# scrolling speed as much as possible. Equation: `speed`×`distance`[px]
# ÷1_000_000_000 = emulated mouse wheel rolls.
speed = 10000000
# Which button toggles scrolling. Button 2 is the middle mouse button.
# Button 3 is upper button on Wacom Bamboo Pen.
button_id = 3

# Click using keyboard. Active only until `timeout_ms` has passed since
# the last time any of grabbed devices was used.
[keyboard_click]
# Delay activation on program startup.
warmup_ms = 500
# How much time must pass until keys go back to normal.
timeout_ms = 1000
# Key that emulates left mouse button.
# **DO NOT USE TOGGLE BUTTON (CapsLock etc.)**
key_lmb = 52 # Z
# Key that emulates right mouse button.
# **DO NOT USE TOGGLE BUTTON (CapsLock etc.)**
key_rmb = 53 # X
# Some unused key. Recommended any of F13-F24.
# **DO NOT USE TOGGLE BUTTON (CapsLock etc.)**
unused_key = "F13"
```

## Example config - mouse

Use config above, but replace `xinput_grep` with f.e. "Gaming Mouse", and `stylus_button_id` with some button ID. Middle button ID is 2, but that requires you to unbind its pasting functionality somehow. I recommend using additional buttons if your mouse has these. Button IDs 4 and 5 are reserved for scroll events. You can check button IDs with `xinput test-xi2 --root`.

## Installation

First, install `xinput`, `xdotool`, `xmodmap`, `grep` and `cut`.

Grab binary from [Releases page](https://github.com/pzmarzly/x11-input-supercharger/releases), or build it yourself by copying the source and running `cargo build --release`, or have Cargo download the sources and put binary in `PATH` for you with `cargo install x11-input-supercharger`. Rust stable toolchain is required.

`Config.toml` must be in current working directory when starting the program.

## Miscellaneous

Entering/leaving scrolling mode changes keymap, which causes lag in Chromium-based browsers. Don't set the timeout too short.

The program grabs root X11 input device.

The code is ugly, and the program sometimes crashes on shutdown (but doesn't seem to leave the system in broken state).

KSysGuard shows the program uses 0-2% of CPU time on Intel i5 6300HQ.