use super::{
    dto::{CreateUserRequest, UserPaginatedQuery},
    entity::prelude::*,
};
use crate::{
    common::{constants::MAX_PER_PAGE, error::ApiError},
    features::user::dto::{UpdateUserRequest, UserResponse},
    utils::{password, uuid::generate_uuidv7},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Select, TryIntoModel,
};
use serde::Deserialize;
use uuid::Uuid;

pub struct UserService {
    db: DatabaseConnection,
}

impl UserService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_all_paginated(
        &self,
        payload: UserPaginatedQuery,
    ) -> Result<(Vec<UserResponse>, u64, u64, u64), ApiError> {
        let page = payload.page.max(1);
        let per_page = payload.per_page.clamp(1, MAX_PER_PAGE);

        let mut query = UserEntity::find();

        if let Some(search) = payload.search.as_deref() {
            let trimmed = search.trim();
            if !trimmed.is_empty() {
                let keyword = format!("%{}%", trimmed.to_lowercase());
                query = query.filter(super::entity::Column::Email.like(keyword));
            }
        }

        query = apply_sort(query, payload.sort_by, payload.sort_order);

        let paginator = query.paginate(&self.db, per_page);

        let total = paginator.num_items().await.map_err(|e| {
            tracing::error!(event = "user.pagination.failed", error = %e);
            ApiError::InternalServerError
        })?;

        let items: Vec<UserModel> = paginator.fetch_page(page - 1).await.map_err(|e| {
            tracing::error!(event = "user.pagination.failed", error = %e);
            ApiError::InternalServerError
        })?;

        let data = items.into_iter().map(UserResponse::from).collect();

        Ok((data, page, per_page, total))
    }

    pub async fn create(&self, data: &CreateUserRequest) -> Result<UserModel, ApiError> {
        let email = self.ensure_email_available(&data.email, None).await?;
        let hashed_password = password::hash(&data.password)?;

        let user = UserActiveModel {
            id: Set(generate_uuidv7()),
            email: Set(email),
            hashed_password: Set(hashed_password),
            ..Default::default()
        };

        user.insert(&self.db).await.map_err(|e| {
            tracing::error!(event = "user.create.failed", error = %e);
            ApiError::InternalServerError
        })
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<UserModel, ApiError> {
        UserEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(event = "user.find_by_id.failed", error = %e);
                ApiError::InternalServerError
            })?
            .ok_or(ApiError::UserNotFound)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserModel>, ApiError> {
        UserEntity::find()
            .filter(super::entity::Column::Email.eq(email.to_lowercase()))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(event = "user.find_by_email.failed", error = %e);
                ApiError::InternalServerError
            })
    }

    pub async fn update(&self, id: Uuid, data: &UpdateUserRequest) -> Result<UserModel, ApiError> {
        let user = self.find_by_id(id).await?;
        let mut active: UserActiveModel = user.into();
        let mut changed = false;

        if let Some(email) = data.email.as_ref() {
            let email = self.ensure_email_available(email, Some(id)).await?;
            active.email = Set(email);
            changed = true;
        }

        if !changed {
            return Ok(active.try_into_model().map_err(|e| {
                tracing::error!(event = "user.update.model_convert.failed", error = %e);
                ApiError::InternalServerError
            })?);
        }

        active.update(&self.db).await.map_err(|e| {
            tracing::error!(event = "user.update.failed", error = %e);
            ApiError::InternalServerError
        })
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ApiError> {
        let result = UserEntity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(event = "user.delete.failed", error = %e);
                ApiError::InternalServerError
            })?;

        if result.rows_affected == 0 {
            return Err(ApiError::UserNotFound);
        }

        Ok(())
    }

    async fn ensure_email_available(
        &self,
        email: &str,
        except_id: Option<Uuid>,
    ) -> Result<String, ApiError> {
        let normalized = email.trim().to_lowercase();

        if let Some(existing) = self.find_by_email(&normalized).await? {
            if except_id != Some(existing.id) {
                return Err(ApiError::EmailAlreadyExists);
            }
        }

        Ok(normalized)
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserSortBy {
    #[default]
    CreatedAt,
    UpdatedAt,
    Email,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserSortOrder {
    Asc,
    #[default]
    Desc,
}

fn apply_sort(
    query: Select<UserEntity>,
    sort_by: Option<UserSortBy>,
    sort_order: Option<UserSortOrder>,
) -> Select<UserEntity> {
    use super::entity::Column;

    let column = match sort_by.unwrap_or_default() {
        UserSortBy::Email => Column::Email,
        UserSortBy::CreatedAt => Column::CreatedAt,
        UserSortBy::UpdatedAt => Column::UpdatedAt,
    };

    match sort_order.unwrap_or_default() {
        UserSortOrder::Asc => query.order_by_asc(column),
        UserSortOrder::Desc => query.order_by_desc(column),
    }
}
