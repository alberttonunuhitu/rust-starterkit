use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::{
    common::error::{ApiError, AppResult},
    features::user::entity::{self, prelude::*},
    utils::{paseto, password},
};

use super::dto::{
    AuthUserInfo, LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse,
};

pub struct AuthService {
    db: DatabaseConnection,
}

impl AuthService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn login(&self, request: &LoginRequest) -> AppResult<LoginResponse> {
        let email = request.email.trim().to_lowercase();

        let user = UserEntity::find()
            .filter(entity::Column::Email.eq(&email))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(event = "auth.login.db_error", error = %e);
                ApiError::InternalServerError
            })?
            .ok_or_else(|| ApiError::Unauthorized)?;

        let password_valid = password::verify(&request.password, &user.hashed_password)?;
        if !password_valid {
            return Err(ApiError::Unauthorized);
        }

        let user_id = user.id;
        let user_email = user.email.clone();

        let access_token = paseto::create_access_token(user_id, &user_email).map_err(|e| {
            tracing::error!(event = "auth.token.create_failed", error = %e);
            ApiError::InternalServerError
        })?;

        let refresh_token = paseto::create_refresh_token(user_id, &user_email).map_err(|e| {
            tracing::error!(event = "auth.token.create_failed", error = %e);
            ApiError::InternalServerError
        })?;

        let hashed_refresh = password::hash(&refresh_token)?;
        let mut active: UserActiveModel = user.into();
        active.current_hashed_refresh_token = Set(Some(hashed_refresh));
        active.update(&self.db).await.map_err(|e| {
            tracing::error!(event = "auth.login.store_token_failed", error = %e);
            ApiError::InternalServerError
        })?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            user: AuthUserInfo {
                id: user_id,
                email: user_email,
            },
        })
    }

    pub async fn refresh_token(
        &self,
        request: &RefreshTokenRequest,
    ) -> AppResult<RefreshTokenResponse> {
        let claims = paseto::verify_refresh_token(&request.refresh_token).map_err(|e| {
            tracing::warn!(event = "auth.refresh.token_invalid", error = %e);
            ApiError::Unauthorized
        })?;

        let sub_str = claims
            .get_claim("sub")
            .and_then(|v| v.as_str())
            .ok_or(ApiError::Unauthorized)?;

        let user_id = Uuid::parse_str(sub_str).map_err(|_| ApiError::Unauthorized)?;

        let user = UserEntity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(event = "auth.refresh.db_error", error = %e);
                ApiError::InternalServerError
            })?
            .ok_or(ApiError::Unauthorized)?;

        let stored_hash = user
            .current_hashed_refresh_token
            .as_deref()
            .ok_or(ApiError::Unauthorized)?;

        let valid = password::verify(&request.refresh_token, stored_hash)?;
        if !valid {
            tracing::warn!(event = "auth.refresh.token_mismatch", %user_id);
            return Err(ApiError::Unauthorized);
        }

        let user_email = user.email.clone();

        let access_token = paseto::create_access_token(user_id, &user_email).map_err(|e| {
            tracing::error!(event = "auth.token.create_failed", error = %e);
            ApiError::InternalServerError
        })?;

        let new_refresh_token =
            paseto::create_refresh_token(user_id, &user_email).map_err(|e| {
                tracing::error!(event = "auth.token.create_failed", error = %e);
                ApiError::InternalServerError
            })?;

        let hashed_new_refresh = password::hash(&new_refresh_token)?;
        let mut active: UserActiveModel = user.into();
        active.current_hashed_refresh_token = Set(Some(hashed_new_refresh));
        active.update(&self.db).await.map_err(|e| {
            tracing::error!(event = "auth.refresh.store_token_failed", error = %e);
            ApiError::InternalServerError
        })?;

        tracing::info!(event = "auth.refresh.success", %user_id);

        Ok(RefreshTokenResponse {
            access_token,
            refresh_token: new_refresh_token,
        })
    }

    pub async fn logout(&self, user_id: Uuid) -> AppResult<()> {
        let user = UserEntity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(event = "auth.logout.db_error", error = %e);
                ApiError::InternalServerError
            })?
            .ok_or(ApiError::Unauthorized)?;

        let mut active: UserActiveModel = user.into();
        active.current_hashed_refresh_token = Set(None);
        active.update(&self.db).await.map_err(|e| {
            tracing::error!(event = "auth.logout.failed", error = %e);
            ApiError::InternalServerError
        })?;

        tracing::info!(event = "auth.logout.success", %user_id);

        Ok(())
    }
}
