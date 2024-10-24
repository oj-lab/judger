use nix::errno::Errno;

pub fn userid(name: &str) -> i32 {
    Errno::set_raw(0);

    let passwd = unsafe { libc::getpwnam(name.as_ptr() as *const i8) };
    if passwd.is_null() || Errno::last_raw() != 0 {
        return -1;
    }

    unsafe { (*passwd).pw_uid as i32 }
}

pub fn groupid(name: &str) -> i32 {
    Errno::set_raw(0);

    let group = unsafe { libc::getgrnam(name.as_ptr() as *const i8) };
    if group.is_null() || Errno::last_raw() != 0 {
        return -1;
    }

    unsafe { (*group).gr_gid as i32 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_userid() {
        assert_eq!(userid("root"), 0);
        assert_eq!(userid("nonexistent"), -1);
    }

    #[test]
    fn test_groupid() {
        assert_eq!(groupid("root"), 0);
        assert_eq!(groupid("nonexistent"), -1);
    }
}
