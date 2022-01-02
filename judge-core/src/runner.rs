use crate::utils::get_default_rusage;
use libc::{c_int, rusage, wait4, WSTOPPED};
use std::process::Command;

pub fn run_process() {
    let child = Command::new("./../infinite_loop")
        .spawn()
        .expect("Failed to execute child");

    let mut status: c_int = 0;
    let mut usage: rusage = get_default_rusage();
    unsafe {
        wait4(child.id() as i32, &mut status, WSTOPPED, &mut usage);
    }
}
