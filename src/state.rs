use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::{
    config::AppConfig,
    features::{auth::service::AuthService, user::service::UserService},
};

pub struct AppServices {
    pub user: Arc<UserService>,
    pub auth: Arc<AuthService>,
}

impl AppServices {
    fn new(db: &DatabaseConnection) -> Self {
        Self {
            user: Arc::new(UserService::new(db.clone())),
            auth: Arc::new(AuthService::new(db.clone())),
        }
    }
}

pub struct AppState {
    pub db: DatabaseConnection,
    pub config: AppConfig,
    pub services: AppServices,
}

impl AppState {
    pub fn new(db: DatabaseConnection, config: AppConfig) -> Self {
        let services = AppServices::new(&db);
        Self {
            db,
            config,
            services,
        }
    }
}
