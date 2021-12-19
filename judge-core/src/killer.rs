use std::time::Duration;

fn kill(pid: u32) {
    unsafe {
        libc::kill(pid as i32, libc::SIGKILL);
    }
}

pub async fn timeout_killer(pid: u32, timeout: u64) {
    use tokio::time::sleep;

    sleep(Duration::from_millis(timeout)).await;

    kill(pid);
}

#[cfg(test)]
mod kill {
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

    #[tokio::test]
    async fn test_timeout_killer() {
        let pid = start_test_process();

        timeout_killer(pid, 5000).await;

        println!("killed pid:{}", pid);
    }
}
