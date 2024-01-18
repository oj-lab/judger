use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "judge-client")]
pub struct JudgeClientOpt {
    #[structopt(long)]
    pub env_path: Option<String>,

    /// Port to listen to
    #[structopt(env = "BASE_URL", default_value = "http://localhost:8080/api/v1/judge")]
    pub base_url: String,

    #[structopt(env = "INTERVAL", default_value = "10")]
    pub interval: i32,
}

pub fn load_option() -> JudgeClientOpt {
    // First load env_path from Args
    let opt = JudgeClientOpt::from_args();
    if let Some(env_path) = opt.env_path {
        dotenv::from_path(env_path).ok();
    } else {
        dotenv::dotenv().ok();
    }

    // Load opt again with ENV
    let opt = JudgeClientOpt::from_args();
    log::debug!("load opt: {:?}", opt);
    opt
}

pub fn setup_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
}
