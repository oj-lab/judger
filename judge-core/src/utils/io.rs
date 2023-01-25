use std::os::fd::RawFd;

use log::{info, debug};

use crate::error::JudgeCoreError;

pub fn write_str_to_fd(fd: RawFd, s: &str, append_eof: bool) -> Result<(), JudgeCoreError> {
    let mut bytes = s.as_bytes().to_vec();
    if append_eof {
        bytes.push(0);
    }
    let len = bytes.len();
    let mut written_bytes = 0;
    while written_bytes < len {
        match nix::unistd::write(fd, &bytes[written_bytes..]) {
            Ok(len) => {
                info!("write {} bytes", len);
                written_bytes += len;
            }
            Err(e) => {
                info!("write error: {:?}", e);
                break;
            }
        }
    }
    Ok(())
}

pub fn read_string_from_fd(fd: RawFd, exit_while_eof: bool) -> Result<String, JudgeCoreError> {
    let mut buffer_vec: Vec<u8> = Vec::new();
    loop {
        let mut buffer: [u8; 1] = [0; 1];
        match nix::unistd::read(fd, &mut buffer) {
            Ok(_len) => {
                // if `exit_while_eof` is set, read until EOF
                if buffer[0] == 0 && exit_while_eof {
                    info!("read exit with EOF");
                    break;
                }
                buffer_vec.push(buffer[0]);
            }
            Err(e) => {
                match e {
                    nix::errno::Errno::EAGAIN => {
                        debug!("read exit with EAGAIN");
                        break;
                    }
                    _ => {}
                }
                info!("read error: {:?}", e);
                break;
            }
        }
    }
    let string = String::from_utf8(buffer_vec)?;
    debug!("read string:\n{}", string);
    Ok(string)
}
