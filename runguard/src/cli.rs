use std::path;

use clap::Parser;

#[derive(Parser)]
#[command(
    override_usage = "runguard [OPTION]... <COMMAND>...",
    about = "Run COMMAND with specified options.",
    after_help = "Note that root privileges are needed for the `root' and `user' options. \
If `user' is set, then `group' defaults to the same to prevent security issues, \
since otherwise the process would retain group root permissions. \
The COMMAND path is relative to the changed ROOT directory if specified. \
TIME may be specified as a float; two floats separated by `:' are treated as soft and hard limits. \
The runtime written to file is that of the last of wall/cpu time options set, \
and defaults to CPU time when neither is set. \
When run setuid without the `user' option, the user ID is set to the real user ID."
)]
pub struct Cli {
    /// run COMMAND with root directory set to ROOT
    #[arg(short, long)]
    root: String,

    /// run COMMAND as user with username or ID USER
    #[arg(short, long)]
    user: String,

    /// run COMMAND under group with name or ID GROUP
    #[arg(short, long)]
    group: String,

    /// change to directory DIR after setting root directory
    #[arg(short = 'd', long, value_name = "DIR")]
    chdir: String,

    /// kill COMMAND after TIME wallclock seconds
    #[arg(short = 't', long, value_name = "TIME")]
    walltime: humantime::Duration,

    /// set maximum CPU time to TIME seconds
    #[arg(short = 'C', long, value_name = "TIME")]
    cputime: humantime::Duration,

    /// set total memory limit to SIZE kB
    #[arg(short = 'm', long, value_name = "SIZE")]
    memsize: u64,

    /// set maximum created filesize to SIZE kB
    #[arg(short = 'f', long, value_name = "SIZE")]
    filesize: u64,

    /// set maximum no. processes to N
    #[arg(short = 'p', long, value_name = "N")]
    nproc: u64,

    /// use only processor number ID (or set, e.g. \"0,2-3\")
    #[arg(short = 'P', long, value_name = "ID")]
    cpuset: String,

    /// disable core dumps
    #[arg(short = 'c', long)]
    no_core: bool,

    /// redirect COMMAND stdout output to FILE
    #[arg(short = 'o', long, value_name = "FILE")]
    stdout: path::PathBuf,

    /// redirect COMMAND stderr output to FILE
    #[arg(short = 'e', long, value_name = "FILE")]
    stderr: path::PathBuf,

    /// truncate COMMAND stdout/stderr streams at SIZE kB
    #[arg(short, long, value_name = "SIZE")]
    streamsize: u64,

    /// preserve environment variables (default only PATH)
    #[arg(short = 'E', long)]
    environment: String,

    /// write metadata (runtime, exitcode, etc.) to FILE
    #[arg(short = 'M', long, value_name = "FILE")]
    metadata: path::PathBuf,

    /// process ID of runpipe to send SIGUSR1 signal when
    /// timelimit is reached
    #[arg(short = 'U', long, value_name = "PID")]
    runpipepid: u32,

    /// display some extra warnings and information
    #[arg(short, long)]
    verbose: bool,

    /// suppress all warnings and verbose output
    #[arg(short, long)]
    quiet: bool,

    /// output version information and exit
    #[arg(long)]
    version: bool,

    #[arg(required = true)]
    command: Vec<String>,
}