use crate::common::pagination::PaginatedQuery;
use crate::features::user::entity::prelude::UserModel;
use crate::features::user::service::{UserSortBy, UserSortOrder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub type UserPaginatedQuery = PaginatedQuery<UserSortBy, UserSortOrder>;

impl From<UserModel> for UserResponse {
    fn from(model: UserModel) -> Self {
        Self {
            id: model.id,
            email: model.email,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
