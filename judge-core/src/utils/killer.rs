use std::{thread::sleep, time::Duration};

use log::{error, info};

pub fn timeout_killer(pid: u32, timeout: u64) {
    sleep(Duration::from_millis(timeout));

    unsafe {
        let return_num = libc::kill(pid as i32, libc::SIGKILL);
        if return_num != 0 {
            error!("killer kill process:{} failed", pid);
            // TODO: retry or do some other things
        } else {
            info!("killer successfully killed process:{}", pid);
        }
    }
}

#[cfg(test)]
mod killer {
    use super::*;

    fn start_test_process() -> u32 {
        use std::process::Command;

        let child = Command::new("./../infinite_loop")
            .spawn()
            .expect("Failed to execute child");

        child.id()
    }

    #[test]
    fn test_timeout_killer() {
        use std::thread;
        let pid = start_test_process();

        thread::spawn(move || timeout_killer(pid, 5000));
        sleep(Duration::from_millis(5000));

        println!("killed pid:{}", pid);
    }
}
