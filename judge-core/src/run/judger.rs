use log::info;

use crate::{
    error::JudgeCoreError,
    utils::io::{read_string_from_fd, write_str_to_fd},
};

use super::RunConfig;

pub fn run_judger(config: RunConfig) -> Result<(), JudgeCoreError> {
    info!("in_fd: {}", config.input_fd);

    if config.program_path.is_none() {
        info!("no program path provided, using default judger");
        default_judger(config)?;
    }

    Ok(())
}

fn default_judger(config: RunConfig) -> Result<(), JudgeCoreError> {
    let input_string = read_string_from_fd(config.input_fd, true)?;
    info!("input:\n{}", input_string);

    // in default case, input transfer to output without any change
    let output_str = &input_string;
    write_str_to_fd(config.output_fd, output_str, false)?;

    Ok(())
}
