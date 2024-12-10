use axum_template::engine::Engine;
use minijinja::Environment;

use crate::DatabaseOrbiter;

type AppEngine = Engine<Environment<'static>>;

#[derive(Clone)]
pub struct AppState {
    pub engine: AppEngine,
    pub orbiter: DatabaseOrbiter,
}
