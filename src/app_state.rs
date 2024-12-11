use crate::DatabaseOrbiter;

#[derive(Clone)]
pub struct AppState {
    pub orbiter: DatabaseOrbiter,
}
