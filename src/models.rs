use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Contact {
    pub id: Uuid,
    pub name: String,
    pub phone: String,
    pub email: String,
}

impl Contact {
    pub fn new(name: String, phone: String, email: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            phone,
            email,
        }
    }
}
