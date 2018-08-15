use std::borrow::Borrow;
use std::io::Read;
use std::process::{Child, ChildStdout, Command, Stdio};
use std::thread::sleep;

use MOMENT;

trait CommandExt {
    fn my_spawn(&mut self) -> Child;
}

trait ChildExt {
    fn my_stdout(self) -> ChildStdout;
}

impl CommandExt for Command {
    fn my_spawn(&mut self) -> Child {
        self.stdout(Stdio::piped()).spawn().unwrap()
    }
}

impl ChildExt for Child {
    fn my_stdout(self) -> ChildStdout {
        self.stdout.unwrap()
    }
}

pub fn list(grep: impl Borrow<str>) -> Vec<String> {
    let grep = grep.borrow();
    let list = Command::new("xinput").arg("list").my_spawn().my_stdout();
    let list = Command::new("grep")
        .arg(grep)
        .stdin(list)
        .my_spawn()
        .my_stdout();
    let list = Command::new("cut")
        .arg("-f2")
        .stdin(list)
        .my_spawn()
        .my_stdout();
    let mut list = Command::new("cut")
        .arg("-f2")
        .arg("-d=")
        .stdin(list)
        .my_spawn();
    while list.try_wait().unwrap().is_none() {
        sleep(MOMENT);
    }
    let mut stdout = String::with_capacity(32);
    list.my_stdout().read_to_string(&mut stdout).unwrap();
    stdout.lines().map(|x| x.to_string()).collect()
}

pub fn test_xi2() -> ChildStdout {
    Command::new("xinput")
        .arg("test-xi2")
        .arg("--root")
        .stderr(Stdio::piped())
        .my_spawn()
        .my_stdout()
}
