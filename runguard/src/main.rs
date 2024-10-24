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
struct Cli {
    /// run COMMAND with root directory set to ROOT
    #[arg(short, long)]
    root: String,

    /// run COMMAND as user with username or ID USER
    #[arg(short, long)]
    user: String,

    /// run COMMAND under group with name or ID GROUP
    #[arg(short, long)]
    group: String,

    #[arg(required = true)]
    command: Vec<String>,
}

fn main() {
    let _ = Cli::parse();
}
