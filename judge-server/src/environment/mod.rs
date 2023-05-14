use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "judge-server")]
pub struct JudgeServerOpt {
    #[structopt(long)]
    pub env_path: Option<String>,

    /// Port to listen to
    #[structopt(env = "PORT", default_value = "8000")]
    pub port: u16,
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
    env_logger::init()
}
