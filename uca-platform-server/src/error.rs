//! 错误类型与响应映射。

use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;
use thiserror::Error;

/// 标准 API 错误响应。
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    /// 面向客户端的稳定错误码。
    pub code: String,
    /// 可读的错误信息。
    pub message: String,
}

/// 应用错误类型。
#[derive(Debug, Error)]
pub enum AppError {
    /// 配置错误（启动期）。
    #[error("configuration error: {0}")]
    Config(String),
    /// 数据库访问错误。
    #[error("database error: {0}")]
    Database(String),
    /// 认证或鉴权错误。
    #[error("auth error: {0}")]
    Auth(String),
    /// 校验错误。
    #[error("validation error: {0}")]
    Validation(String),
    /// 资源不存在。
    #[error("not found: {0}")]
    NotFound(String),
    /// 请求参数错误。
    #[error("bad request: {0}")]
    BadRequest(String),
    /// 内部错误。
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// 创建配置错误。
    pub fn config(message: &str) -> Self {
        Self::Config(message.to_string())
    }

    /// 创建校验错误。
    pub fn validation(message: &str) -> Self {
        Self::Validation(message.to_string())
    }

    /// 创建请求参数错误。
    pub fn bad_request(message: &str) -> Self {
        Self::BadRequest(message.to_string())
    }

    /// 创建认证/鉴权错误。
    pub fn auth(message: &str) -> Self {
        Self::Auth(message.to_string())
    }

    /// 创建资源不存在错误。
    pub fn not_found(message: &str) -> Self {
        Self::NotFound(message.to_string())
    }

    /// 创建内部错误。
    pub fn internal(message: &str) -> Self {
        Self::Internal(message.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "config_error"),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error"),
            AppError::Auth(_) => (StatusCode::UNAUTHORIZED, "auth_error"),
            AppError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, "validation_error"),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };

        let body = ErrorBody {
            code: code.to_string(),
            message: self.to_string(),
        };
        (status, Json(body)).into_response()
    }
}
