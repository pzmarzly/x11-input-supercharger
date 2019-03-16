use const_cstr::const_cstr;
use x11::xinput::XListInputDevices;
use x11::xinput2::*;
use x11::xlib::*;

use std::collections::HashSet;
use std::ffi::CStr;
use std::mem::uninitialized;

pub struct XLib {
    s: XLibLocked,
}

pub struct XLibLocked {
    display: *mut Display,
    devices: Vec<Device>,
    ev: XEvent,
    future_grabs: HashSet<i32>,
    root: u64,
    xi_opcode: i32,
}

pub struct Device {
    pub id: u32,
    pub name: String,
}

const_cstr! {
    X_INPUT_EXTENSION_STR = "XInputExtension";
}

impl XLib {
    pub fn new() -> Self {
        unsafe {
            let display = XOpenDisplay(0 as *const _);
            if display == (0 as *mut _) {
                panic!("Cannot connect to Xserver");
            }

            let mut xi_opcode = 0;
            let mut event = 0;
            let mut error = 0;
            XQueryExtension(
                display,
                X_INPUT_EXTENSION_STR.as_ptr(),
                &mut xi_opcode,
                &mut event,
                &mut error,
            );

            let mut devices_num = 0;
            let devices_raw = XListInputDevices(display, &mut devices_num);
            let devices_num = devices_num as usize;
            // Vec will also deallocate output of XListInputDevices!
            let devices = Vec::from_raw_parts(devices_raw, devices_num, devices_num);
            let devices = devices
                .into_iter()
                .map(|d| Device {
                    id: d.id as u32,
                    name: CStr::from_ptr(d.name).to_string_lossy().into_owned(),
                })
                .collect();

            Self {
                s: XLibLocked {
                    display,
                    devices,
                    ev: uninitialized(),
                    future_grabs: HashSet::with_capacity(8),
                    root: XDefaultRootWindow(display),
                    xi_opcode,
                },
            }
        }
    }
    pub fn grab(&mut self, events: &[i32]) {
        events.iter().for_each(|x| {
            self.s.future_grabs.insert(*x);
        });
    }
    pub fn list_devices(&self) -> impl Iterator<Item = &Device> {
        self.s.devices.iter()
    }
    pub fn get_device_id(&self, device_name: &str, subdevice: u32) -> Option<u32> {
        self.list_devices()
            .filter(|x| x.name == device_name)
            .skip(subdevice as usize)
            .next()
            .map(|x| x.id)
    }
    pub fn get_keys(&mut self, key: u8) -> Vec<u64> {
        unsafe {
            let mut len = 0;
            let keysyms = XGetKeyboardMapping(self.s.display, key, 1, &mut len);
            Vec::from_raw_parts(keysyms, len as usize, len as usize)
        }
    }
    pub fn finish(mut self) -> XLibLocked {
        unsafe {
            let mut mask = vec![0u8; 4];
            self.s
                .future_grabs
                .drain()
                .for_each(|event| XISetMask(&mut mask, event));
            let mut ev_mask = XIEventMask {
                deviceid: XIAllMasterDevices,
                mask_len: mask.len() as i32,
                mask: mask.as_mut_ptr(),
            };
            XISelectEvents(self.s.display, self.s.root, &mut ev_mask, 1);
            XSync(self.s.display, 0);
            self.s
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Event {
    pub kind: i32,
    pub source_id: u32,
    pub detail: u8,
}

impl XLibLocked {
    pub fn poll(&mut self) -> Option<Event> {
        unsafe {
            let mut result = None;
            XNextEvent(self.display, &mut self.ev);
            if XGetEventData(self.display, &mut self.ev.generic_event_cookie) != 0 {
                if self.ev.generic_event_cookie.type_ == GenericEvent
                    && self.ev.generic_event_cookie.extension == self.xi_opcode
                {
                    #[allow(non_upper_case_globals)]
                    match self.ev.generic_event_cookie.evtype {
                        XI_DeviceChanged | XI_HierarchyChanged | XI_Enter | XI_Leave
                        | XI_FocusIn | XI_FocusOut | XI_PropertyEvent => {
                            println!("Warning: Unsupported event received!")
                        }
                        XI_RawKeyPress | XI_RawKeyRelease | XI_RawButtonPress
                        | XI_RawButtonRelease | XI_RawMotion | XI_RawTouchBegin
                        | XI_RawTouchUpdate | XI_RawTouchEnd => {
                            let data = self.ev.generic_event_cookie.data as *mut XIRawEvent;
                            result = Some(Event {
                                kind: self.ev.generic_event_cookie.evtype,
                                source_id: (*data).sourceid as u32,
                                detail: (*data).detail as u8,
                            });
                        }
                        _ => {
                            let data = self.ev.generic_event_cookie.data as *mut XIDeviceEvent;
                            result = Some(Event {
                                kind: self.ev.generic_event_cookie.evtype,
                                source_id: (*data).sourceid as u32,
                                detail: (*data).detail as u8,
                            });
                        }
                    }
                }
                XFreeEventData(self.display, &mut self.ev.generic_event_cookie);
            }
            result
        }
    }
}
