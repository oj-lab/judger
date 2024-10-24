use std::{fmt::Arguments, fs::File, io::Write, thread::sleep};

use chrono::{format, DateTime, Local};
use libc::{tms, _SC_CLK_TCK};
use nix::{
    errno::Errno,
    sys::signal::{
        sigprocmask, SigSet,
        SigmaskHow::SIG_BLOCK,
        Signal::{SIGALRM, SIGTERM},
    },
};

use crate::{
    cgroup::{cgroup_strerror_safe, CGroupError},
    safe_libc::{fclose, strerror, sysconf},
    PROGNAME,
};

pub struct Context {
    pub use_walltime: bool,
    pub use_user: bool,
    pub use_group: bool,

    pub progstarttime: DateTime<Local>,
    pub endtime: DateTime<Local>,
    pub starttime: DateTime<Local>,

    pub startticks: tms,
    pub endticks: tms,

    pub received_signal: i32, // default -1

    pub outputmeta: bool,
    pub metafile: Option<File>,
    pub metafilename: String,

    pub in_error_handling: bool,

    pub be_quiet: bool,
    pub be_verbose: bool,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            use_walltime: false,
            use_user: false,
            use_group: false,
            progstarttime: chrono::Local::now(),
            endtime: chrono::Local::now(),
            starttime: chrono::Local::now(),
            startticks: unsafe {
                let mut ticks = std::mem::zeroed();
                libc::times(&mut ticks);
                ticks
            },
            endticks: unsafe {
                let mut ticks = std::mem::zeroed();
                libc::times(&mut ticks);
                ticks
            },
            received_signal: -1,
            outputmeta: true,
            metafile: Some(File::create("metafile.txt").unwrap()),
            metafilename: "metafile.txt".to_string(),
            in_error_handling: false,
            be_quiet: false,
            be_verbose: true,
        }
    }
}

impl Context {
    pub fn warning(&self, format: Arguments) {
        if !self.be_quiet {
            eprintln!("{}: warning: {}", PROGNAME, format);
        }
    }

    pub fn verbose(&self, format: Arguments) {
        if !self.be_quiet && self.be_verbose {
            let currtime = chrono::Local::now();
            let runtime =
                (currtime - self.progstarttime).num_microseconds().unwrap() as f64 / 1_000_000.0;
            eprintln!(
                "{} [{} @ {:10.6}]: verbose: {}",
                PROGNAME,
                std::process::id(),
                runtime,
                format
            );
        }
    }

    pub fn error(&mut self, mut errnum: i32, format: Arguments) {
        // Silently ignore errors that happen while handling other errors.
        if self.in_error_handling {
            return;
        }
        self.in_error_handling = true;

        /*
         * Make sure the signal handler for these (terminate()) does not
         * interfere, we are exiting now anyway.
         */
        let mut sigs: SigSet = SigSet::empty();
        sigs.add(SIGALRM);
        sigs.add(SIGTERM);
        let _ = sigprocmask(SIG_BLOCK, Some(&sigs), None);

        /* First print to string to be able to reuse the message. */
        let mut errstr: String = PROGNAME.to_string();
        if !format.to_string().is_empty() {
            errstr = format!("{}: {}", errstr, strerror(errnum));
        }
        if errnum != 0 {
            /* Special case libcgroup error codes. */
            if errnum == CGroupError::ECGOTHER as i32 {
                errstr = format!("{}: libcgroup", errstr);
                errnum = Errno::last_raw();
            }
            if errnum == CGroupError::ECGROUPNOTCOMPILED as i32 {
                errstr = format!("{}: {}", errstr, cgroup_strerror_safe(errnum));
            } else {
                errstr = format!("{}: {}", errstr, strerror(errnum));
            }
        }
        if format.to_string().is_empty() && errnum == 0 {
            errstr = format!("{}: unknown error", errstr);
        }

        self.write_meta("internal-error", format_args!("{}", errstr));
        if self.outputmeta && self.metafile.is_some() {
            if let Some(file_ref) = &self.metafile {
                if fclose(file_ref.try_clone().unwrap()) != 0 {
                    eprintln!("\nError closing metafile '{}'.\n", self.metafilename);
                }
            }
        }

        eprintln!(
            "{}\nTry `{} --help' for more information.",
            errstr, PROGNAME
        );
    }

    pub fn write_meta(&mut self, key: &str, format: Arguments) {
        if !self.outputmeta {
            return;
        }

        if let Some(file) = self.metafile.as_mut() {
            if writeln!(file, "{}: {}", key, format).is_err() {
                self.outputmeta = false;
                self.error(0, format_args!("cannot write to file: {}", "metafile.txt"));
            }
        } else {
            self.outputmeta = false;
            self.error(0, format_args!("cannot write to file: {}", "metafile.txt"));
        }
    }

    pub fn output_exit_time(&mut self, exitcode: i32, cpudiff: f64) {
        self.verbose(format_args!("command exited with exitcode {}", exitcode));
        self.write_meta("exitcode", format_args!("{}", exitcode));

        if self.received_signal != -1 {
            let received_signal = self.received_signal;
            self.write_meta("signal", format_args!("{}", received_signal));
        }

        let walldiff =
            (self.endtime - self.starttime).num_microseconds().unwrap() as f64 / 1_000_000.0;

        let ticks_per_second = sysconf(_SC_CLK_TCK);
        let userdiff = (self.endticks.tms_cutime as f64 - self.startticks.tms_cutime as f64)
            / ticks_per_second as f64;
        let systemdiff = (self.endticks.tms_cstime as f64 - self.startticks.tms_cstime as f64)
            / ticks_per_second as f64;

        self.write_meta("wall-time", format_args!("{:.3}", walldiff));
        self.write_meta("user-time", format_args!("{:.3}", userdiff));
        self.write_meta("sys-time", format_args!("{:.3}", systemdiff));
        self.write_meta("cpu-time", format_args!("{:.3}", cpudiff));

        self.verbose(format_args!(
            "runtime is {:.3} seconds real, {:.3} seconds user, {:.3} seconds sys",
            walldiff, userdiff, systemdiff
        ));
    }
}

#[test]
fn test_context() {
    let mut ctx = Context {
        ..Default::default()
    };

    ctx.error(0, format_args!("test error"));
    ctx.write_meta("test", format_args!("test meta"));
    ctx.warning(format_args!("test warning"));
    ctx.verbose(format_args!("test verbose"));
    sleep(std::time::Duration::from_secs(1));
    ctx.verbose(format_args!("test verbose"));
}
