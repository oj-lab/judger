use std::sync::RwLock;

use actix_web::{get, web, HttpResponse};
use lazy_static::lazy_static;

use crate::error::ServiceError;

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

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_state))]
pub struct StateApiDoc;

pub fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/state").service(get_state));
}

#[utoipa::path(
    context_path = "/api/v1/state",     
    responses(
        (status = 200, description = "Judge run successfully")
    )
)]
#[get("")]
pub async fn get_state() -> Result<HttpResponse, ServiceError> {
    let state = STATE.read().map_err(|e| {
        ServiceError::InternalError(anyhow::anyhow!("Failed to lock state: {:?}", e))
    })?;
    Ok(HttpResponse::Ok().content_type(
        "application/text; charset=utf-8",
    ).body(format!("{:?}", *state)))
}
