use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, EntityTrait, Set};
use sea_orm_migration::MigratorTrait;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;
use tower::util::ServiceExt;
use url::Url;
use uuid::Uuid;
use volunteerhours::{
    auth::{encrypt_secret, generate_recovery_codes, generate_session_token, generate_totp, hash_session_token},
    config::Config,
    entities::{
        recovery_codes, sessions, students, totp_secrets, users,
    },
    migration::Migrator,
    routes,
    state::AppState,
};
use webauthn_rs::prelude::WebauthnBuilder;

struct TestContext {
    app: axum::Router,
    state: AppState,
    _tempdir: Option<TempDir>,
}

async fn setup_context() -> TestContext {
    let (db, tempdir) = setup_database().await;
    Migrator::up(&db, None).await.expect("migrate");

    let config = Config {
        bind_addr: "127.0.0.1:0".to_string(),
        developer_mode: true,
        allow_http: true,
        database_url: database_url(),
        rp_id: "localhost".to_string(),
        rp_origin: Url::parse("http://localhost:8443").unwrap(),
        tls_cert_path: "data/tls/cert.pem".into(),
        tls_key_path: "data/tls/key.enc".into(),
        tls_import_cert_path: None,
        tls_import_key_path: None,
        tls_key_enc_key: vec![0u8; 32],
        upload_dir: "data/uploads".into(),
        session_cookie_name: "vh_session".to_string(),
        session_ttl_seconds: 3600,
        auth_secret_key: vec![1u8; 32],
        bootstrap_token: None,
    };

    let mut builder = WebauthnBuilder::new(&config.rp_id, &config.rp_origin).unwrap();
    builder = builder.rp_name("VolunteerHours");
    let webauthn = builder.build().unwrap();

    let state = AppState::new(Arc::new(config), db, webauthn).unwrap();
    let app = routes::router(state.clone());

    TestContext {
        app,
        state,
        _tempdir: tempdir,
    }
}

async fn setup_database() -> (DatabaseConnection, Option<TempDir>) {
    let url = database_url();
    if url.starts_with("sqlite:") {
        let tempdir = TempDir::new().expect("tempdir");
        let db_path = tempdir.path().join("test.db");
        let sqlite_url = format!("sqlite://{}?mode=rwc", db_path.display());
        let db = Database::connect(sqlite_url).await.expect("connect sqlite");
        (db, Some(tempdir))
    } else {
        let db = Database::connect(url).await.expect("connect db");
        (db, None)
    }
}

fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/test.db?mode=rwc".to_string())
}

async fn reset_database(state: &AppState) {
    let tables = [
        "form_field_values",
        "form_fields",
        "review_signatures",
        "attachments",
        "contest_records",
        "volunteer_records",
        "competition_library",
        "recovery_codes",
        "totp_secrets",
        "passkeys",
        "sessions",
        "devices",
        "students",
        "users",
    ];
    let backend = state.db.get_database_backend();
    match backend {
        sea_orm::DatabaseBackend::MySql => {
            state
                .db
                .execute(sea_orm::Statement::from_string(
                    backend,
                    "SET FOREIGN_KEY_CHECKS=0".to_string(),
                ))
                .await
                .expect("disable fk");
            for table in tables {
                state
                    .db
                    .execute(sea_orm::Statement::from_string(
                        backend,
                        format!("DELETE FROM {table}"),
                    ))
                    .await
                    .expect("delete");
            }
            state
                .db
                .execute(sea_orm::Statement::from_string(
                    backend,
                    "SET FOREIGN_KEY_CHECKS=1".to_string(),
                ))
                .await
                .expect("enable fk");
        }
        sea_orm::DatabaseBackend::Postgres => {
            let joined = tables.join(", ");
            state
                .db
                .execute(sea_orm::Statement::from_string(
                    backend,
                    format!("TRUNCATE TABLE {joined} RESTART IDENTITY CASCADE"),
                ))
                .await
                .expect("truncate");
        }
        sea_orm::DatabaseBackend::Sqlite => {
            state
                .db
                .execute(sea_orm::Statement::from_string(
                    backend,
                    "PRAGMA foreign_keys = OFF".to_string(),
                ))
                .await
                .expect("disable fk");
            for table in tables {
                state
                    .db
                    .execute(sea_orm::Statement::from_string(
                        backend,
                        format!("DELETE FROM {table}"),
                    ))
                    .await
                    .expect("delete");
            }
            state
                .db
                .execute(sea_orm::Statement::from_string(
                    backend,
                    "PRAGMA foreign_keys = ON".to_string(),
                ))
                .await
                .expect("enable fk");
        }
    }
}

async fn create_user(state: &AppState, username: &str, role: &str) -> users::Model {
    let now = chrono::Utc::now();
    let id = Uuid::new_v4();
    let model = users::ActiveModel {
        id: Set(id),
        username: Set(username.to_string()),
        display_name: Set(username.to_string()),
        role: Set(role.to_string()),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    };
    users::Entity::insert(model)
        .exec_without_returning(&state.db)
        .await
        .expect("insert user");
    users::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .expect("find user")
        .expect("user exists")
}

async fn create_student(state: &AppState, student_no: &str) -> students::Model {
    let now = chrono::Utc::now();
    let id = Uuid::new_v4();
    let model = students::ActiveModel {
        id: Set(id),
        student_no: Set(student_no.to_string()),
        name: Set("张三".to_string()),
        gender: Set("男".to_string()),
        department: Set("信息学院".to_string()),
        major: Set("软件工程".to_string()),
        class_name: Set("软工1班".to_string()),
        phone: Set("13800000000".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    students::Entity::insert(model)
        .exec_without_returning(&state.db)
        .await
        .expect("insert student");
    students::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .expect("find student")
        .expect("student exists")
}

async fn create_session_cookie(state: &AppState, user_id: Uuid) -> String {
    let token = generate_session_token();
    let token_hash = hash_session_token(&token);
    let now = chrono::Utc::now();
    let id = Uuid::new_v4();
    let model = sessions::ActiveModel {
        id: Set(id),
        user_id: Set(user_id),
        token_hash: Set(token_hash),
        expires_at: Set(now + chrono::Duration::seconds(state.config.session_ttl_seconds)),
        created_at: Set(now),
        last_seen_at: Set(Some(now)),
    };
    sessions::Entity::insert(model)
        .exec_without_returning(&state.db)
        .await
        .expect("insert session");
    format!("{}={}", state.config.session_cookie_name, token)
}

fn json_request(method: &str, path: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn multipart_request(path: &str, filename: &str, bytes: Vec<u8>) -> Request<Body> {
    let boundary = "----volunteerhoursboundary";
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        b"Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet\r\n\r\n",
    );
    body.extend_from_slice(&bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    Request::builder()
        .method("POST")
        .uri(path)
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(body))
        .unwrap()
}

fn build_xlsx(headers: &[&str], rows: &[Vec<&str>]) -> Vec<u8> {
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let worksheet = workbook.add_worksheet();
    for (idx, header) in headers.iter().enumerate() {
        worksheet.write_string(0, idx as u16, *header).unwrap();
    }
    for (row_idx, row) in rows.iter().enumerate() {
        for (col, value) in row.iter().enumerate() {
            worksheet.write_string((row_idx + 1) as u32, col as u16, *value).unwrap();
        }
    }
    workbook.save_to_buffer().unwrap()
}

#[tokio::test]
async fn health_and_bootstrap() {
    let ctx = setup_context().await;
    reset_database(&ctx.state).await;

    let response = ctx
        .app
        .clone()
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request = json_request(
        "POST",
        "/auth/bootstrap",
        json!({ "username": "admin", "display_name": "管理员" }),
    );
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request = json_request("POST", "/auth/passkey/login/start", json!({ "username": "missing" }));
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let request = json_request(
        "POST",
        "/auth/passkey/login/finish",
        json!({ "session_id": Uuid::new_v4(), "credential": {} }),
    );
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn admin_competitions_and_forms() {
    let ctx = setup_context().await;
    reset_database(&ctx.state).await;

    let admin = create_user(&ctx.state, "admin1", "admin").await;
    let cookie = create_session_cookie(&ctx.state, admin.id).await;

    let form_field = json!({
        "form_type": "volunteer",
        "field_key": "location",
        "label": "地点",
        "field_type": "text",
        "required": true,
        "order_index": 1
    });
    let request = json_request("POST", "/admin/form-fields", form_field)
        .with_cookie(&cookie);
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request = Request::builder()
        .method("GET")
        .uri("/auth/me")
        .header(header::COOKIE, cookie.clone())
        .body(Body::empty())
        .unwrap();
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request = Request::builder()
        .method("GET")
        .uri("/forms/volunteer/fields")
        .header(header::COOKIE, cookie.clone())
        .body(Body::empty())
        .unwrap();
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn import_students_and_records() {
    let ctx = setup_context().await;
    reset_database(&ctx.state).await;
    let admin = create_user(&ctx.state, "admin2", "admin").await;
    let cookie = create_session_cookie(&ctx.state, admin.id).await;

    let competitions_xlsx = build_xlsx(&["竞赛名称"], &[vec!["全国大学生数学建模竞赛"]]);
    let request = multipart_request("/admin/competitions/import", "competitions.xlsx", competitions_xlsx)
        .with_cookie(&cookie);
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let student_xlsx = build_xlsx(
        &["学号", "姓名", "性别", "院系", "专业", "班级", "手机号"],
        &[vec!["2023001", "张三", "男", "信息学院", "软件工程", "软工1班", "13800000000"]],
    );
    let request = multipart_request("/students/import", "students.xlsx", student_xlsx)
        .with_cookie(&cookie);
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let volunteer_xlsx = build_xlsx(
        &["学号", "标题", "描述", "自评学时"],
        &[vec!["2023001", "社区服务", "清洁", "4"]],
    );
    let request = multipart_request("/admin/records/volunteer/import", "volunteer.xlsx", volunteer_xlsx)
        .with_cookie(&cookie);
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let contest_xlsx = build_xlsx(
        &["学号", "竞赛名称", "获奖等级", "自评学时"],
        &[vec!["2023001", "全国大学生数学建模竞赛", "省赛一等奖", "8"]],
    );
    let request = multipart_request("/admin/records/contest/import", "contest.xlsx", contest_xlsx)
        .with_cookie(&cookie);
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn create_and_review_records() {
    let ctx = setup_context().await;
    reset_database(&ctx.state).await;

    let student_user = create_user(&ctx.state, "2023002", "student").await;
    create_student(&ctx.state, "2023002").await;
    let student_cookie = create_session_cookie(&ctx.state, student_user.id).await;

    let now = chrono::Utc::now();
    let field_id = Uuid::new_v4();
    let field_model = volunteerhours::entities::form_fields::ActiveModel {
        id: Set(field_id),
        form_type: Set("volunteer".to_string()),
        field_key: Set("location".to_string()),
        label: Set("地点".to_string()),
        field_type: Set("text".to_string()),
        required: Set(true),
        order_index: Set(1),
        created_at: Set(now),
        updated_at: Set(now),
    };
    volunteerhours::entities::form_fields::Entity::insert(field_model)
        .exec_without_returning(&ctx.state.db)
        .await
        .unwrap();

    let request = json_request(
        "POST",
        "/records/volunteer",
        json!({ "title": "志愿活动", "description": "服务", "self_hours": 3, "custom_fields": {} }),
    )
    .with_cookie(&student_cookie);
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_ne!(response.status(), StatusCode::OK);

    let request = json_request(
        "POST",
        "/records/volunteer",
        json!({ "title": "志愿活动", "description": "服务", "self_hours": 3, "custom_fields": { "location": "校内操场" } }),
    )
    .with_cookie(&student_cookie);
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request = json_request(
        "POST",
        "/records/contest",
        json!({
            "contest_name": "全国大学生数学建模竞赛",
            "award_level": "省赛一等奖",
            "self_hours": 8,
            "custom_fields": {}
        }),
    )
    .with_cookie(&student_cookie);
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let reviewer = create_user(&ctx.state, "reviewer1", "reviewer").await;
    let reviewer_cookie = create_session_cookie(&ctx.state, reviewer.id).await;

    let volunteer_record = volunteerhours::entities::VolunteerRecord::find()
        .one(&ctx.state.db)
        .await
        .unwrap()
        .unwrap();
    let request = json_request(
        "POST",
        &format!("/records/volunteer/{}/review", volunteer_record.id),
        json!({ "stage": "first", "hours": 2, "status": "approved", "rejection_reason": null }),
    )
    .with_cookie(&reviewer_cookie);
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn export_endpoints() {
    let ctx = setup_context().await;
    reset_database(&ctx.state).await;
    let admin = create_user(&ctx.state, "admin3", "admin").await;
    let cookie = create_session_cookie(&ctx.state, admin.id).await;

    let student_user = create_user(&ctx.state, "2023003", "student").await;
    create_student(&ctx.state, "2023003").await;
    let student_cookie = create_session_cookie(&ctx.state, student_user.id).await;

    let request = json_request(
        "POST",
        "/records/volunteer",
        json!({ "title": "志愿活动", "description": "服务", "self_hours": 2, "custom_fields": {} }),
    )
    .with_cookie(&student_cookie);
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request = json_request(
        "POST",
        "/export/summary/excel",
        json!({ "department": "信息学院" }),
    )
    .with_cookie(&cookie);
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request = Request::builder()
        .method("POST")
        .uri("/export/student/2023003/excel")
        .header(header::COOKIE, cookie.clone())
        .body(Body::empty())
        .unwrap();
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let record = volunteerhours::entities::VolunteerRecord::find()
        .one(&ctx.state.db)
        .await
        .unwrap()
        .unwrap();
    let request = Request::builder()
        .method("POST")
        .uri(format!("/export/record/volunteer/{}/pdf", record.id))
        .header(header::COOKIE, cookie.clone())
        .body(Body::empty())
        .unwrap();
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn upload_attachments_and_signatures() {
    let ctx = setup_context().await;
    reset_database(&ctx.state).await;

    let student_user = create_user(&ctx.state, "2023010", "student").await;
    create_student(&ctx.state, "2023010").await;
    let student_cookie = create_session_cookie(&ctx.state, student_user.id).await;

    let request = json_request(
        "POST",
        "/records/volunteer",
        json!({ "title": "志愿活动", "description": "服务", "self_hours": 2, "custom_fields": {} }),
    )
    .with_cookie(&student_cookie);
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let record = volunteerhours::entities::VolunteerRecord::find()
        .one(&ctx.state.db)
        .await
        .unwrap()
        .unwrap();

    let attachment = multipart_request(
        &format!("/attachments/volunteer/{}", record.id),
        "proof.txt",
        b"test".to_vec(),
    )
    .with_cookie(&student_cookie);
    let response = ctx.app.clone().oneshot(attachment).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let reviewer = create_user(&ctx.state, "reviewer2", "reviewer").await;
    let reviewer_cookie = create_session_cookie(&ctx.state, reviewer.id).await;
    let signature = multipart_request(
        &format!("/signatures/volunteer/{}/first", record.id),
        "sig.png",
        b"sig".to_vec(),
    )
    .with_cookie(&reviewer_cookie);
    let response = ctx.app.clone().oneshot(signature).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_totp_and_recovery() {
    let ctx = setup_context().await;
    reset_database(&ctx.state).await;
    let user = create_user(&ctx.state, "2023999", "student").await;

    let (secret, _) = generate_totp("VolunteerHours", &user.username).unwrap();
    let encrypted = encrypt_secret(&secret, &ctx.state.config.auth_secret_key).unwrap();
    let totp_id = Uuid::new_v4();
    let totp_model = totp_secrets::ActiveModel {
        id: Set(totp_id),
        user_id: Set(user.id),
        secret_enc: Set(encrypted),
        enabled: Set(true),
        verified_at: Set(Some(chrono::Utc::now())),
        created_at: Set(chrono::Utc::now()),
    };
    totp_secrets::Entity::insert(totp_model)
        .exec_without_returning(&ctx.state.db)
        .await
        .unwrap();

    let code = totp_rs::TOTP::new_unchecked(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        secret.clone(),
        Some(user.username.clone()),
        "VolunteerHours".to_string(),
    )
    .generate_current()
    .unwrap();

    let request = json_request(
        "POST",
        "/auth/totp/verify",
        json!({ "username": user.username, "code": code }),
    );
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let codes = generate_recovery_codes(1).unwrap();
    let recovery = &codes[0];
    let recovery_id = Uuid::new_v4();
    let recovery_model = recovery_codes::ActiveModel {
        id: Set(recovery_id),
        user_id: Set(user.id),
        code_hash: Set(recovery.hash.clone()),
        used_at: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    recovery_codes::Entity::insert(recovery_model)
        .exec_without_returning(&ctx.state.db)
        .await
        .unwrap();

    let request = json_request(
        "POST",
        "/auth/recovery/verify",
        json!({ "username": user.username, "code": recovery.plain }),
    );
    let response = ctx.app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

trait WithCookie {
    fn with_cookie(self, cookie: &str) -> Request<Body>;
}

impl WithCookie for Request<Body> {
    fn with_cookie(self, cookie: &str) -> Request<Body> {
        let (parts, body) = self.into_parts();
        let mut builder = Request::builder()
            .method(parts.method)
            .uri(parts.uri)
            .version(parts.version);
        for (key, value) in parts.headers.iter() {
            builder = builder.header(key, value);
        }
        builder.header(header::COOKIE, cookie).body(body).unwrap()
    }
}
