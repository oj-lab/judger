use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "judge-server")]
pub struct JudgeServerOpt {
    #[structopt(long)]
    pub env_path: Option<String>,

    /// Port to listen to
    #[structopt(env = "PORT", default_value = "8080")]
    pub port: u16,

    #[structopt(long, default_value = "data/dev-problem-package")]
    pub problem_package_dir: PathBuf,

    #[structopt(long, default_value = "data/rclone.conf")]
    pub rclone_config: PathBuf,

    #[structopt(env = "BASE_URL", default_value = "http://localhost:8080/api/v1/judge")]
    pub base_url: String,

    #[structopt(env = "INTERVAL", default_value = "10")]
    pub interval: i32,
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
