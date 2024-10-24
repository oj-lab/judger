use std::ffi::CString;
use std::fs::File;
use std::io::{self, BufRead};
use std::os::raw::c_char;

use crate::context::Context;

extern "C" {
    fn cgroup_new_cgroup(name: *const c_char) -> *mut libc::c_void;
    fn cgroup_strerror(err: i32) -> *const c_char;
}

pub enum CGroupError {
    ECGROUPNOTCOMPILED = 50000,
    ECGROUPNOTMOUNTED,
    ECGROUPNOTEXIST,
    ECGROUPNOTCREATED,
    ECGROUPSUBSYSNOTMOUNTED,
    ECGROUPNOTOWNER,
    /** Controllers bound to different mount points */
    ECGROUPMULTIMOUNTED,
    /* This is the stock error. Default error. @todo really? */
    ECGROUPNOTALLOWED,
    ECGMAXVALUESEXCEEDED,
    ECGCONTROLLEREXISTS,
    ECGVALUEEXISTS,
    ECGINVAL,
    ECGCONTROLLERCREATEFAILED,
    ECGFAIL,
    ECGROUPNOTINITIALIZED,
    ECGROUPVALUENOTEXIST,
    /**
     * Represents error coming from other libraries like glibc. @c libcgroup
     * users need to check cgroup_get_last_errno() upon encountering this
     * error.
     */
    ECGOTHER,
    ECGROUPNOTEQUAL,
    ECGCONTROLLERNOTEQUAL,
    /** Failed to parse rules configuration file. */
    ECGROUPPARSEFAIL,
    /** Rules list does not exist. */
    ECGROUPNORULES,
    ECGMOUNTFAIL,
    /**
     * Not an real error, it just indicates that iterator has come to end
     * of sequence and no more items are left.
     */
    ECGEOF = 50023,
    /** Failed to parse config file (cgconfig.conf). */
    ECGCONFIGPARSEFAIL,
    ECGNAMESPACEPATHS,
    ECGNAMESPACECONTROLLER,
    ECGMOUNTNAMESPACE,
    ECGROUPUNSUPP,
    ECGCANTSETVALUE,
    /** Removing of a group failed because it was not empty. */
    ECGNONEMPTY,
}

struct CGroup {
    ctx: Context,
    cgroup: *mut libc::c_void,
}

impl CGroup {
    fn new(mut ctx: Context, name: &str) -> Self {
        let cgroup_name = CString::new(name).expect("CString::new failed");
        unsafe {
            let cgroup = cgroup_new_cgroup(cgroup_name.as_ptr());
            if cgroup.is_null() {
                ctx.error(0, format_args!("cgroup_new_cgroup"));
            } else {
                ctx.verbose(format_args!("cgroup_new_cgroup: {}", name));
            }
            CGroup { ctx, cgroup }
        }
    }
}

fn cgroup_is_v2() -> bool {
    let file = match File::open("/proc/mounts") {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Error opening /proc/mounts");
            return false;
        }
    };

    let reader = io::BufReader::new(file);
    for line in reader.lines() {
        if let Ok(line) = line {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[1] == "/sys/fs/cgroup" && parts[2] == "cgroup2" {
                return true;
            }
        }
    }

    false
}

pub fn cgroup_strerror_safe(err: i32) -> String {
    unsafe {
        let errstr = cgroup_strerror(err);
        let errstr = std::ffi::CStr::from_ptr(errstr).to_str().unwrap();
        errstr.to_string()
    }
}

#[test]
fn test_cgroup() {
    let ctx = Context::default();
    let _ = CGroup::new(ctx, "my_cgroup");

    if cgroup_is_v2() {
        println!("cgroup v2 is enabled");
    } else {
        println!("cgroup v2 is not enabled");
    }
}

#[test]
fn test_cgroup_strerror() {
    println!(
        "{}",
        cgroup_strerror_safe(CGroupError::ECGROUPNOTCOMPILED as i32)
    );
}
