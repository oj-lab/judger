use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "judger")]
pub struct JudgeServerOpt {
    /// For loading Opt from .env file
    #[structopt(long)]
    pub env_path: Option<String>,

    /// Port to listen to
    #[structopt(env = "PORT", default_value = "8080")]
    pub port: u16,

    // TODO: make rclone optional
    #[structopt(long, default_value = "data/default-rclone.conf")]
    pub rclone_config: PathBuf,
    #[structopt(long, default_value = "oj-lab-problem-package")]
    pub problem_package_bucket: String,
    /// Where to store problem package
    #[structopt(long, default_value = "data/problem-package")]
    pub problem_package_dir: PathBuf,

    #[structopt(env = "PLATFORM_URI", default_value = "http://localhost:8080/")]
    pub platform_uri: String,
    /// Interval to fetch task in seconds
    #[structopt(env = "FETCH_TASK_INTERVAL", default_value = "10")]
    pub fetch_task_interval: u64,
}

pub fn load_option() -> JudgeServerOpt {
    // First load env_path from Args
    let opt = JudgeServerOpt::from_args();
    if let Some(env_path) = opt.env_path {
        dotenv::from_path(env_path).ok();
    } else {
        dotenv::dotenv().ok();
    }

    // Load opt again with ENV
    let opt = JudgeServerOpt::from_args();
    log::debug!("load opt: {:?}", opt);
    opt
}

pub fn setup_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
}
