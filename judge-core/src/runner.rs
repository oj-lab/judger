use crate::utils::get_default_rusage;
use libc::{c_int, rusage, wait4, WSTOPPED};
use std::process::Command;
use nix::sys::resource::{
    setrlimit,
    Resource::{
        RLIMIT_STACK,
        RLIMIT_AS,
        RLIMIT_CPU,
        RLIMIT_NPROC,
        RLIMIT_FSIZE,
    },
};

pub fn run_process() {
    // TODO: Handle error
    setrlimit(RLIMIT_STACK, Some(1024*1024*1024), Some(1024*1024*1024)).unwrap();
    setrlimit(RLIMIT_AS, Some(1024*1024*1024), Some(1024*1024*1024)).unwrap();
    setrlimit(RLIMIT_CPU, Some(2), Some(2)).unwrap();
    setrlimit(RLIMIT_NPROC, None, None).unwrap();
    setrlimit(RLIMIT_FSIZE, Some(1024*1024*1024), Some(1024*1024*1024)).unwrap();

    let child = Command::new("./../infinite_loop")
        .spawn()
        .expect("Failed to execute child");

    let mut status: c_int = 0;
    let mut usage: rusage = get_default_rusage();
    unsafe {
        wait4(child.id() as i32, &mut status, WSTOPPED, &mut usage);
    }
}