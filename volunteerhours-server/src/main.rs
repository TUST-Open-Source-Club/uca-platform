//! VolunteerHours 服务端入口。

use std::net::SocketAddr;
use std::sync::Arc;

use axum::http::HeaderValue;
use axum_server::tls_rustls::RustlsConfig;
use sea_orm_migration::MigratorTrait;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{fmt, EnvFilter};
use webauthn_rs::prelude::WebauthnBuilder;

use volunteerhours::{
    config::Config,
    db,
    error::AppError,
    migration::Migrator,
    routes,
    state::AppState,
    tls,
};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Arc::new(Config::from_env()?);

    if config.developer_mode {
        tls::ensure_tls_material(&config)?;
    }

    let db = db::connect(&config.database_url).await?;
    Migrator::up(&db, None)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut builder = WebauthnBuilder::new(&config.rp_id, &config.rp_origin)
        .map_err(|err| AppError::internal(&format!("webauthn config error: {err}")))?;
    builder = builder.rp_name("VolunteerHours");
    let webauthn = builder
        .build()
        .map_err(|err| AppError::internal(&format!("webauthn build error: {err}")))?;

    let state = AppState::new(config.clone(), db, webauthn)?;

    let origin = HeaderValue::from_str(config.rp_origin.as_str())
        .map_err(|_| AppError::internal("invalid RP_ORIGIN header"))?;
    let cors = CorsLayer::new()
        .allow_origin(origin)
        .allow_credentials(true)
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ])
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::DELETE,
        ]);

    let app = routes::router(state).layer(cors);

    let addr: SocketAddr = config
        .bind_addr
        .parse()
        .map_err(|_| AppError::config("BIND_ADDR invalid"))?;

    if config.allow_http {
        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .map_err(|err| AppError::internal(&format!("server error: {err}")))?;
    } else {
        tls::ensure_tls_material(&config)?;
        let (cert_pem, key_pem) = tls::load_tls_pem(&config)?;
        let tls_config = RustlsConfig::from_pem(cert_pem, key_pem)
            .await
            .map_err(|err| AppError::internal(&format!("failed to configure TLS: {err}")))?;

        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service())
            .await
            .map_err(|err| AppError::internal(&format!("server error: {err}")))?;
    }

    Ok(())
}
