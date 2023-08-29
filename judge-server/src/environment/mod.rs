use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "judge-server")]
pub struct JudgeServerOpt {
    #[structopt(long)]
    pub env_path: Option<String>,

    /// Port to listen to
    #[structopt(env = "PORT", default_value = "8000")]
    pub port: u16,

    #[structopt(long, default_value = "dev-problem-package")]
    pub problem_package_dir: PathBuf,
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
