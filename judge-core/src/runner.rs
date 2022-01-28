use crate::rules::{cpp_loader::CppLoader, get_default_kill_context, load_rules};
use nix::{
    errno::Errno,
    sys::resource::{
        setrlimit,
        Resource::{RLIMIT_AS, RLIMIT_CPU, RLIMIT_FSIZE, RLIMIT_NPROC, RLIMIT_STACK},
    },
    unistd::dup2,
    unistd::execve,
};
use std::ffi::CString;
use std::fs::File;
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};

pub fn run_process() {
    // TODO: Handle error
    set_limit().unwrap();

    let input_file = File::open("../tmp/in").unwrap();
    let output_file = File::options()
        .write(true)
        .truncate(true) // Overwrite the whole content of this file
        .open("../tmp/out")
        .unwrap();

    let input_raw_fd: RawFd = input_file.as_raw_fd();
    let stdin_raw_fd: RawFd = io::stdin().as_raw_fd();
    dup2(input_raw_fd, stdin_raw_fd).unwrap();
    let output_raw_fd: RawFd = output_file.as_raw_fd();
    let stdout_raw_fd: RawFd = io::stdout().as_raw_fd();
    dup2(output_raw_fd, stdout_raw_fd).unwrap();

    load_rules(Box::new(CppLoader {
        ctx: get_default_kill_context().unwrap(),
    }))
    .unwrap();

    execve(
        &CString::new("./../read_and_write").expect("CString::new failed"),
        &[CString::new("").expect("CString::new failed")],
        &[CString::new("").expect("CString::new failed")],
    )
    .unwrap();
}

fn set_limit() -> Result<(), Errno> {
    setrlimit(
        RLIMIT_STACK,
        Some(1024 * 1024 * 1024),
        Some(1024 * 1024 * 1024),
    )?;
    setrlimit(
        RLIMIT_AS,
        Some(1024 * 1024 * 1024),
        Some(1024 * 1024 * 1024),
    )?;
    setrlimit(RLIMIT_CPU, Some(6), Some(6))?;
    setrlimit(RLIMIT_NPROC, None, None)?;
    setrlimit(
        RLIMIT_FSIZE,
        Some(1024 * 1024 * 1024),
        Some(1024 * 1024 * 1024),
    )?;
    Ok(())
}
