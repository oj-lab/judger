// ignore unused warning
#![allow(dead_code)]

use std::ffi::CString;
use std::fs::File;
use std::io::{self, BufRead};
use std::os::raw::c_char;

extern "C" {
    fn cgroup_new_cgroup(name: *const c_char) -> *mut libc::c_void;
    fn cgroup_add_controller(
        cgroup: *mut libc::c_void,
        controller: *const c_char,
    ) -> *mut libc::c_void;
    fn cgroup_add_value_string(
        controller: *mut libc::c_void,
        name: *const c_char,
        value: *const c_char,
    ) -> i32;
    fn cgroup_get_value_string(
        controller: *mut libc::c_void,
        name: *const c_char,
        value: *mut *mut c_char,
    ) -> i32;
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
    cgroup: *mut libc::c_void,
}

impl CGroup {
    fn new(name: &str) -> Self {
        let cgroup_name = CString::new(name).expect("CString::new failed");
        unsafe {
            let cgroup = cgroup_new_cgroup(cgroup_name.as_ptr());
            if cgroup.is_null() {
                panic!("Failed to create cgroup");
            }
            CGroup { cgroup }
        }
    }

    fn add_controller(&self, controller: &str) -> *mut libc::c_void {
        let controller = CString::new(controller).expect("CString::new failed");
        unsafe {
            let ret = cgroup_add_controller(self.cgroup, controller.as_ptr());
            if ret.is_null() {
                eprintln!("Failed to add controller to cgroup");
            }
            ret
        }
    }

    fn add_value_string(&self, controller: *mut libc::c_void, name: &str, value: &str) {
        let name = CString::new(name).expect("CString::new failed");
        let value = CString::new(value).expect("CString::new failed");
        unsafe {
            let ret = cgroup_add_value_string(controller, name.as_ptr(), value.as_ptr());
            if ret != 0 {
                eprintln!(
                    "Failed to add value to cgroup: {}",
                    cgroup_strerror_safe(ret)
                );
            }
        }
    }

    fn cgroup_get_value_string(&self, controller: *mut libc::c_void, name: &str) -> String {
        let name = CString::new(name).expect("CString::new failed");
        let mut value: *mut c_char = std::ptr::null_mut();
        unsafe {
            let ret = cgroup_get_value_string(controller, name.as_ptr(), &mut value);
            if ret != 0 {
                eprintln!(
                    "Failed to get value from cgroup: {}",
                    cgroup_strerror_safe(ret)
                );
            }
            let value = std::ffi::CStr::from_ptr(value).to_str().unwrap();
            value.to_string()
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
    for line in reader.lines().flatten() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[1] == "/sys/fs/cgroup" && parts[2] == "cgroup2" {
            return true;
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
    let cgroup = CGroup::new("my_cgroup");

    if cgroup_is_v2() {
        println!("cgroup v2 is enabled");
    } else {
        println!("cgroup v2 is not enabled");
    }

    let controller = cgroup.add_controller("cpuset");
    cgroup.add_value_string(controller, "cpuset.cpus", "0-1");
    let value = cgroup.cgroup_get_value_string(controller, "cpuset.cpus");
    println!("{}", value);
}

#[test]
fn test_cgroup_strerror() {
    println!(
        "{}",
        cgroup_strerror_safe(CGroupError::ECGROUPNOTCOMPILED as i32)
    );
}
