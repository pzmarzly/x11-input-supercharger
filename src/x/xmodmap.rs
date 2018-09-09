use std::io::Write;
use std::process::{Command, Stdio};

pub struct Transaction<'a> {
    items: Vec<Item<'a>>,
}

struct Item<'a> {
    key: u8,
    keysyms: &'a [u64],
}

pub fn transaction() -> Transaction<'static> {
    Transaction {
        items: Vec::with_capacity(2),
    }
}

impl<'a> Transaction<'a> {
    pub fn bind(&mut self, key: u8, keysyms: &'a [u64]) {
        self.items.push(Item { key, keysyms });
    }
    pub fn commit(self) {
        let mut tmp = String::with_capacity(128);
        for item in self.items {
            let mut keysyms_tmp = String::with_capacity(64);
            for keysym in item.keysyms {
                if keysym != &0 {
                    keysyms_tmp += &keysym.to_string();
                } else {
                    keysyms_tmp += "NoSymbol";
                }
                keysyms_tmp += " ";
            }
            tmp += "keycode ";
            tmp += &item.key.to_string();
            tmp += " = ";
            tmp += &keysyms_tmp;
            tmp += "\n";
        }
        map(&tmp.into_bytes());
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
