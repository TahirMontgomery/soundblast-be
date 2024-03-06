use crate::service::Services;

#[derive(Clone)]
pub struct GlobalState {
    pub services: Services,
}
