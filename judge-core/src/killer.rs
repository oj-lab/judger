use std::{thread::sleep, time::Duration};

fn kill(pid: u32) {
    unsafe {
        libc::kill(pid as i32, libc::SIGKILL);
    }
}

pub fn timeout_killer(pid: u32, timeout: u64) {
    sleep(Duration::from_millis(timeout));

    kill(pid);
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
    fn test_kill() {
        let pid = start_test_process();

        kill(pid);

        println!("killed pid:{}", pid);
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
