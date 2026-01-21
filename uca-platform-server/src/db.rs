//! 数据库连接辅助。

use sea_orm::{Database, DatabaseConnection};

use crate::error::AppError;

/// 使用提供的 URL 连接数据库。
pub async fn connect(database_url: &str) -> Result<DatabaseConnection, AppError> {
    Database::connect(database_url)
        .await
        .map_err(|err| AppError::Database(err.to_string()))
}
