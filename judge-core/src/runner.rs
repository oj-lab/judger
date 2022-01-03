use nix::{
    sys::resource::{
        setrlimit,
        Resource::{RLIMIT_AS, RLIMIT_CPU, RLIMIT_FSIZE, RLIMIT_NPROC, RLIMIT_STACK},
    },
    unistd::execve,
};
use std::ffi::CString;

pub fn run_process() {
    // TODO: Handle error
    setrlimit(
        RLIMIT_STACK,
        Some(1024 * 1024 * 1024),
        Some(1024 * 1024 * 1024),
    )
    .unwrap();
    setrlimit(
        RLIMIT_AS,
        Some(1024 * 1024 * 1024),
        Some(1024 * 1024 * 1024),
    )
    .unwrap();
    setrlimit(RLIMIT_CPU, Some(6), Some(6)).unwrap();
    setrlimit(RLIMIT_NPROC, None, None).unwrap();
    setrlimit(
        RLIMIT_FSIZE,
        Some(1024 * 1024 * 1024),
        Some(1024 * 1024 * 1024),
    )
    .unwrap();

    execve(
        &CString::new("./../infinite_loop").expect("CString::new failed"),
        &[CString::new("").expect("CString::new failed")],
        &[CString::new("").expect("CString::new failed")],
    )
    .unwrap();
}
