use std::process::Command;

pub fn need_dep(name: &str) {
    Command::new(name)
        .arg("--version")
        .output()
        .unwrap_or_else(|_| panic!("Missing global binary: {}", name));
}
