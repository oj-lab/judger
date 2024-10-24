use clap::Parser;
use libc::{rlimit, setrlimit};
use nix::errno::Errno;
use nix::unistd::getpid;

mod cgroup;
mod cli;
mod context;
mod safe_libc;
mod types;
mod utils;

const PROGNAME: &str = "runguard";

fn main() {
    let userregex = regex::Regex::new(r"^[A-Za-z][A-Za-z0-9\\._-]*$").unwrap();

    let mut ctx = context::Context::default();
    let cli = cli::Cli::parse();

    if let Some(user) = cli.user {
        ctx.use_user = true;
    }

    ctx.verbose(format_args!("starting in verbose mode, PID = {}", getpid()));

    /* Make sure that we change from group root if we change to an
    unprivileged user to prevent unintended permissions. */
}

fn setrestrictions(ctx: &mut context::Context) {
    let mut lim = rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };

    macro_rules! setlim {
        ($type:ident) => {
            let resource = match stringify!($type) {
                "AS" => libc::RLIMIT_AS,
                "CPU" => libc::RLIMIT_CPU,
                "DATA" => libc::RLIMIT_DATA,
                "FSIZE" => libc::RLIMIT_FSIZE,
                "NPROC" => libc::RLIMIT_NPROC,
                "STACK" => libc::RLIMIT_STACK,
                _ => unreachable!(),
            };

            if unsafe { setrlimit(resource, &lim) } != 0 {
                if Errno::last() == Errno::EPERM {
                    ctx.warning(format_args!(
                        "no permission to set resource RLIMIT_{}",
                        stringify!($type)
                    ));
                } else {
                    ctx.error(
                        Errno::last_raw(),
                        format_args!("setting resource RLIMIT_{}", stringify!($type)),
                    );
                }
            } else {
                ctx.verbose(format_args!(
                    "set RLIMIT_{} with cur = {}, max = {}",
                    stringify!($type),
                    lim.rlim_cur,
                    lim.rlim_max
                ));
            }
        };
    }

    lim.rlim_cur = libc::RLIM_INFINITY;
    lim.rlim_max = libc::RLIM_INFINITY;
    setlim!(AS);
    setlim!(DATA);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setrestrictions() {
        let mut ctx = context::Context::default();
        ctx.be_verbose = true;
        ctx.verbose(format_args!("test_setrestrictions"));
        setrestrictions(&mut ctx);
    }
}
