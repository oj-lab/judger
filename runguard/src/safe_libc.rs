use std::{fs::File, os::fd::IntoRawFd};

pub fn strerror(errnum: i32) -> String {
    unsafe {
        let errstr = libc::strerror(errnum);
        let errstr = std::ffi::CStr::from_ptr(errstr).to_str().unwrap();
        errstr.to_string()
    }
}

pub fn fclose(file: File) -> i32 {
    let fd = file.into_raw_fd();
    unsafe { libc::close(fd) }
}

pub fn sysconf(name: i32) -> i64 {
    unsafe { libc::sysconf(name) }
}

#[test]
fn test_strerror() {
    println!("{}", strerror(libc::EINVAL));
}

#[test]
fn test_fclose() {
    let file = File::open("/dev/null").unwrap();
    assert_eq!(fclose(file), 0);
}

#[test]
fn test_sysconf() {
    let ticks_per_second = sysconf(libc::_SC_CLK_TCK);
    println!("{}", ticks_per_second);
    assert!(ticks_per_second > 0);
}
