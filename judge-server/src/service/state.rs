use std::sync::RwLock;

use lazy_static::lazy_static;

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Idle,
    Busy,
}

lazy_static! {
    pub static ref STATE: RwLock<State> = RwLock::new(State::Idle);
}

pub fn set_busy() -> anyhow::Result<()> {
    log::info!("Trying to set busy");
    let mut state = STATE
        .try_write()
        .map_err(|e| anyhow::anyhow!("Failed to lock state: {:?}", e))?;
    log::info!("State: {:?}", *state);
    if *state == State::Busy {
        anyhow::bail!("Judge server is busy")
    }
    *state = State::Busy;

    Ok(())
}

pub fn set_idle() {
    let mut state = STATE.write().unwrap();
    *state = State::Idle;
}
