use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "judger")]
pub struct JudgeServerOpt {
    /// For loading Opt from .env file
    #[structopt(long, default_value = ".env")]
    pub env_path: PathBuf,
    #[structopt(long, default_value = "override.env")]
    pub override_env_path: PathBuf,

    /// Port to listen to
    #[structopt(env = "PORT", default_value = "8000")]
    pub port: u16,

    #[structopt(long, env = "ENABLE_RCLONE")]
    pub enable_rclone: bool,
    #[structopt(env = "RCLONE_CONFIG_PATH", default_value = "rclone.conf")]
    pub rclone_config_path: PathBuf,
    #[structopt(long, default_value = "oj-lab-problem-package")]
    pub problem_package_bucket: String,

    /// Where to store problem package
    #[structopt(env="PROBLEM_PACKAGE_PATH", default_value = "problem-package")]
    pub problem_package_dir: PathBuf,

    #[structopt(env = "PLATFORM_URI", default_value = "http://localhost:8080/")]
    pub platform_uri: String,
    /// Interval to fetch task in seconds
    #[structopt(env = "FETCH_TASK_INTERVAL", default_value = "10")]
    pub fetch_task_interval: u64,
}

/// Try to load env from a .env file, if not found, fallback to ENV
pub fn load_option() -> JudgeServerOpt {
    println!("PWD: {:?}", std::env::current_dir().unwrap());
    // First load env_path from Args
    let opt = JudgeServerOpt::from_args();
    if opt.env_path.exists() {
        println!("loading env from file: {:?}", opt.env_path);
        dotenv::from_path(opt.env_path).ok();
    } else {
        println!("loading env from ENV");
        dotenv::dotenv().ok();
    }
    if opt.override_env_path.exists() {
        println!(
            "loading override env from file: {:?}",
            opt.override_env_path
        );
        dotenv::from_path(opt.override_env_path).ok();
    }

    setup_logger();

    // Load opt again with ENV
    let opt = JudgeServerOpt::from_args();
    log::debug!("load opt: {:?}", opt);
    opt
}

fn setup_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
}
