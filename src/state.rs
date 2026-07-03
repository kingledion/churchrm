use std::sync::{Arc, RwLock};

use crate::models::Contact;

#[derive(Clone)]
pub struct AppState {
    pub contacts: Arc<RwLock<Vec<Contact>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            contacts: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
