use std::path::PathBuf;

use chrono::Local;
use std::io::Write;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "judger")]
pub struct JudgerOpt {
    /// For loading Opt from .env file
    #[structopt(long, default_value = ".env")]
    pub env_path: PathBuf,
    #[structopt(long, default_value = "override.env")]
    pub override_env_path: PathBuf,

    #[structopt(subcommand)]
    pub cmd: JudgerCommad,

    #[structopt(long, env = "ENABLE_RCLONE")]
    pub enable_rclone: bool,
    #[structopt(env = "RCLONE_CONFIG_PATH", default_value = "rclone.conf")]
    pub rclone_config_path: PathBuf,
    #[structopt(long, default_value = "oj-lab-problem-package")]
    pub problem_package_bucket: String,
    /// Where to store problem package
    #[structopt(env = "PROBLEM_PACKAGE_PATH", default_value = "problem-packages")]
    pub problem_package_dir: PathBuf,
}

#[derive(StructOpt, Debug, Clone)]
pub enum JudgerCommad {
    /// Run as a Judger server which fetch tasks from platform
    Serve {
        #[structopt(env = "PLATFORM_URI", default_value = "http://localhost:8080/")]
        platform_uri: String,
        #[structopt(env = "INTERNAL_TOKEN", default_value = "internal_token")]
        internal_token: String,
        /// Interval to fetch task in seconds
        #[structopt(env = "FETCH_TASK_INTERVAL", default_value = "10")]
        fetch_task_interval: u64,
        #[structopt(env = "PORT", default_value = "8000")]
        port: u16,
    },
    /// Runs a single judge task through command line
    Judge {
        #[structopt(short, long)]
        problem_slug: String,
        #[structopt(short, long)]
        language: judge_core::compiler::Language,
        #[structopt(short, long)]
        src_path: PathBuf,
    },
}

/// Try to load env from a .env file, if not found, fallback to ENV
pub fn load_option() -> JudgerOpt {
    println!("PWD: {:?}", std::env::current_dir().unwrap());
    // First load env_path from Args
    let opt = JudgerOpt::from_args();
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
    let opt = JudgerOpt::from_args();
    log::debug!("load opt: {:?}", opt);
    opt
}

fn setup_logger() {
    let env = env_logger::Env::default().default_filter_or("debug");
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {:5} [{}:{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.file().unwrap(),
                // record.module_path().unwrap_or("<unnamed>"),
                record.line().unwrap(),
                &record.args()
            )
        })
        .init();
}
