use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: i64,
}

impl User {
    pub fn new(email: impl Into<String>, password_hash: impl Into<String>) -> Self {
        Self {
            user_id: Uuid::new_v4().to_string(),
            email: email.into(),
            password_hash: password_hash.into(),
            created_at: Utc::now().timestamp(),
        }
    }
}
