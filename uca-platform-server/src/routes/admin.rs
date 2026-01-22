//! 管理员维护接口。

use axum::{extract::{State, Multipart, Path}, Json};
use axum_extra::extract::cookie::CookieJar;
use calamine::{Data, Reader};
use chrono::{Duration as ChronoDuration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use uuid::Uuid;
use validator::Validate;

use crate::{
    access::{require_role, require_session_user},
    auth::{generate_token, hash_password, hash_token},
    entities::{
        attachments, auth_resets, competition_library, contest_records, form_field_values, form_fields,
        import_template_fields, import_templates, invites, review_signatures, students, users,
        Attachment, CompetitionLibrary, ContestRecord, FormField, FormFieldValue,
        ImportTemplate, ImportTemplateField, ReviewSignature, Student, User,
    },
    error::AppError,
    labor_hours::{load_labor_hour_rules, upsert_labor_hour_rules, LaborHourRuleConfig},
    mailer::send_mail,
    policy::{load_password_policy, upsert_password_policy},
    state::AppState,
    templates::{
        load_export_template, load_import_template, map_import_fields, upsert_export_template,
        ExportTemplateConfig, ImportFieldConfig, ImportTemplateConfig,
    },
};

/// 竞赛库新增请求。
#[derive(Debug, Deserialize, Validate)]
pub struct CreateCompetitionRequest {
    /// 竞赛名称。
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    /// 竞赛年份。
    pub year: Option<i32>,
    /// 竞赛类型（A/B）。
    pub category: Option<String>,
}

/// 竞赛库响应。
#[derive(Debug, Serialize)]
pub struct CompetitionResponse {
    /// 记录 ID。
    pub id: Uuid,
    /// 竞赛年份。
    pub year: Option<i32>,
    /// 竞赛类型。
    pub category: Option<String>,
    /// 竞赛名称。
    pub name: String,
}

/// 劳动学时规则请求。
#[derive(Debug, Deserialize)]
pub struct LaborHourRuleRequest {
    pub base_hours_a: i32,
    pub base_hours_b: i32,
    pub national_leader_hours: i32,
    pub national_member_hours: i32,
    pub provincial_leader_hours: i32,
    pub provincial_member_hours: i32,
    pub school_leader_hours: i32,
    pub school_member_hours: i32,
}

/// 新建用户请求。
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    /// 用户名（学号/工号）。
    #[validate(length(min = 1, max = 64))]
    pub username: String,
    /// 展示名。
    #[validate(length(min = 1, max = 64))]
    pub display_name: String,
    /// 角色（student/teacher/reviewer/admin）。
    #[validate(length(min = 1, max = 16))]
    pub role: String,
    /// 邮箱（非学生必须提供）。
    #[validate(email)]
    pub email: Option<String>,
}

/// 新建用户响应。
#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    /// 用户 ID。
    pub user_id: Option<Uuid>,
    /// 是否已发送邀请邮件。
    pub invite_sent: bool,
}

/// 密码策略配置请求。
#[derive(Debug, Deserialize)]
pub struct PasswordPolicyRequest {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_symbol: bool,
}

/// 密码策略配置响应。
#[derive(Debug, Serialize)]
pub struct PasswordPolicyResponse {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_symbol: bool,
}

/// 导入模板字段请求。
#[derive(Debug, Deserialize, Validate)]
pub struct ImportTemplateFieldRequest {
    /// 字段 key。
    #[validate(length(min = 1, max = 64))]
    pub field_key: String,
    /// 字段标签。
    #[validate(length(min = 1, max = 64))]
    pub label: String,
    /// Excel 表头。
    #[validate(length(max = 128))]
    pub column_title: String,
    /// 是否必填。
    pub required: bool,
    /// 排序序号。
    pub order_index: i32,
    /// 字段说明。
    pub description: Option<String>,
}

/// 导入模板更新请求。
#[derive(Debug, Deserialize, Validate)]
pub struct ImportTemplateRequest {
    /// 模板名称。
    #[validate(length(min = 1, max = 64))]
    pub name: String,
    /// 字段列表。
    #[validate(length(min = 1))]
    pub fields: Vec<ImportTemplateFieldRequest>,
}

/// 导入模板响应。
#[derive(Debug, Serialize)]
pub struct ImportTemplateResponse {
    pub template_key: String,
    pub name: String,
    pub fields: Vec<ImportTemplateFieldResponse>,
}

/// 导入模板字段响应。
#[derive(Debug, Serialize)]
pub struct ImportTemplateFieldResponse {
    pub field_key: String,
    pub label: String,
    pub column_title: String,
    pub required: bool,
    pub order_index: i32,
    pub description: Option<String>,
}

/// 导出模板请求。
#[derive(Debug, Deserialize, Validate)]
pub struct ExportTemplateRequest {
    /// 模板名称。
    #[validate(length(min = 1, max = 64))]
    pub name: String,
    /// 布局 JSON。
    pub layout: serde_json::Value,
}

/// 导出模板响应。
#[derive(Debug, Serialize)]
pub struct ExportTemplateResponse {
    pub template_key: String,
    pub name: String,
    pub layout: serde_json::Value,
}

/// 重置认证方式请求。
#[derive(Debug, Deserialize)]
pub struct ResetUserRequest {
    pub username: String,
}

/// 生成一次性重置码请求。
#[derive(Debug, Deserialize)]
pub struct ResetCodeRequest {
    pub username: String,
    /// 重置目的（password/totp/passkey）。
    pub purpose: String,
}

/// 一次性重置码响应。
#[derive(Debug, Serialize)]
pub struct ResetCodeResponse {
    /// 重置码（仅在 code 模式返回）。
    pub code: Option<String>,
    /// 过期分钟数。
    pub expires_in_minutes: i64,
}

const INVITE_TTL_HOURS: i64 = 72;
const RESET_TTL_MINUTES: i64 = 30;

const COMPETITION_HEADER: [&str; 2] = ["竞赛名称", "name"];
const CONTEST_BASE_HEADERS: [(&str, &[&str]); 6] = [
    ("student_no", &["学号", "student_no"]),
    ("contest_name", &["竞赛名称", "contest_name"]),
    ("award_level", &["获奖等级", "award_level"]),
    ("self_hours", &["自评学时", "self_hours"]),
    ("first_review_hours", &["初审学时", "first_review_hours"]),
    ("final_review_hours", &["复审学时", "final_review_hours"]),
];
const STATUS_HEADERS: [&str; 2] = ["审核状态", "status"];
const REJECTION_HEADERS: [&str; 2] = ["不通过原因", "rejection_reason"];
const IMPORT_TEMPLATE_KEYS: [&str; 3] = ["competition_library", "students", "contest_records"];
const EXPORT_TEMPLATE_KEYS: [&str; 1] = ["labor_hours"];

/// 查询竞赛库。
pub async fn list_competitions(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<CompetitionResponse>>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let items = CompetitionLibrary::find()
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(
        items
            .into_iter()
            .map(|item| CompetitionResponse {
                id: item.id,
                year: item.year,
                category: item.category,
                name: item.name,
            })
            .collect(),
    ))
}

/// 竞赛库公开读取（无需登录）。
pub async fn list_competitions_public(
    State(state): State<AppState>,
) -> Result<Json<Vec<CompetitionResponse>>, AppError> {
    let items = CompetitionLibrary::find()
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(
        items
            .into_iter()
            .map(|item| CompetitionResponse {
                id: item.id,
                year: item.year,
                category: item.category,
                name: item.name,
            })
            .collect(),
    ))
}

/// 新增竞赛名称。
pub async fn create_competition(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<CreateCompetitionRequest>,
) -> Result<Json<CompetitionResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;
    payload
        .validate()
        .map_err(|_| AppError::validation("invalid competition payload"))?;

    let exists = CompetitionLibrary::find()
        .filter(competition_library::Column::Name.eq(&payload.name))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    if exists.is_some() {
        return Err(AppError::bad_request("competition exists"));
    }

    let now = Utc::now();
    let id = Uuid::new_v4();
    let name = payload.name;
    let model = competition_library::ActiveModel {
        id: Set(id),
        year: Set(payload.year),
        category: Set(payload.category.map(|value| value.to_uppercase())),
        name: Set(name.clone()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    competition_library::Entity::insert(model)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(CompetitionResponse {
        id,
        year: payload.year,
        category: payload.category.map(|value| value.to_uppercase()),
        name,
    }))
}

/// 管理员创建用户或发送邀请。
pub async fn create_user(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, AppError> {
    let admin = require_session_user(&state, &jar).await?;
    require_role(&admin, "admin")?;

    payload
        .validate()
        .map_err(|_| AppError::validation("invalid user payload"))?;

    let role = payload.role.as_str();
    if !matches!(role, "student" | "teacher" | "reviewer" | "admin") {
        return Err(AppError::validation("invalid role"));
    }

    if role == "student" {
        let _student = Student::find()
            .filter(students::Column::StudentNo.eq(&payload.username))
            .one(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?
            .ok_or_else(|| AppError::bad_request("student not found"))?;

        let now = Utc::now();
        let default_password = format!("st{}", payload.username);
        let hash = hash_password(&default_password)?;

        if let Some(existing) = User::find()
            .filter(users::Column::Username.eq(&payload.username))
            .one(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?
        {
            let missing_password = existing.password_hash.is_none();
            let mut active: users::ActiveModel = existing.into();
            active.display_name = Set(payload.display_name.clone());
            active.role = Set("student".to_string());
            if payload.email.is_some() {
                active.email = Set(payload.email.clone());
            }
            if missing_password {
                active.password_hash = Set(Some(hash));
            }
            active.allow_password_login = Set(true);
            active.updated_at = Set(now);
            let model = active
                .update(&state.db)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?;
            return Ok(Json(CreateUserResponse {
                user_id: Some(model.id),
                invite_sent: false,
            }));
        }

        let user_id = Uuid::new_v4();
        let model = users::ActiveModel {
            id: Set(user_id),
            username: Set(payload.username.clone()),
            display_name: Set(payload.display_name.clone()),
            role: Set("student".to_string()),
            email: Set(payload.email.clone()),
            password_hash: Set(Some(hash)),
            allow_password_login: Set(true),
            password_updated_at: Set(Some(now)),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
        };
        users::Entity::insert(model)
            .exec_without_returning(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;

        return Ok(Json(CreateUserResponse {
            user_id: Some(user_id),
            invite_sent: false,
        }));
    }

    let email = payload
        .email
        .clone()
        .ok_or_else(|| AppError::validation("email required"))?;
    let base_url = state
        .config
        .base_url
        .as_ref()
        .ok_or_else(|| AppError::config("BASE_URL is required"))?;
    let mail_config = state
        .config
        .mail
        .as_ref()
        .ok_or_else(|| AppError::config("mail config required"))?;

    let existing = User::find()
        .filter(users::Column::Username.eq(&payload.username))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    if existing.is_some() {
        return Err(AppError::bad_request("user already exists"));
    }

    let token = generate_token();
    let token_hash = hash_token(&token);
    let now = Utc::now();
    let expires_at = now + ChronoDuration::hours(INVITE_TTL_HOURS);
    let invite_id = Uuid::new_v4();
    let invite = invites::ActiveModel {
        id: Set(invite_id),
        token_hash: Set(token_hash),
        email: Set(email.clone()),
        username: Set(payload.username.clone()),
        display_name: Set(payload.display_name.clone()),
        role: Set(payload.role.clone()),
        expires_at: Set(expires_at),
        created_at: Set(now),
        used_at: Set(None),
    };
    invites::Entity::insert(invite)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let link = format!("{}/invite?token={}", base_url, token);
    let body = format!(
        "您好，\n\n您被邀请加入 Labor Hours Platform，请点击以下链接完成注册并绑定 TOTP 或 Passkey：\n{}\n\n该链接 {} 小时后失效。",
        link, INVITE_TTL_HOURS
    );
    send_mail(mail_config, &email, "账号邀请", &body).await?;

    Ok(Json(CreateUserResponse {
        user_id: None,
        invite_sent: true,
    }))
}

/// 获取密码策略配置。
pub async fn get_password_policy(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<PasswordPolicyResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;
    let policy = load_password_policy(&state).await?;
    Ok(Json(PasswordPolicyResponse {
        min_length: policy.min_length,
        require_uppercase: policy.require_uppercase,
        require_lowercase: policy.require_lowercase,
        require_digit: policy.require_digit,
        require_symbol: policy.require_symbol,
    }))
}

/// 更新密码策略配置。
pub async fn update_password_policy(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<PasswordPolicyRequest>,
) -> Result<Json<PasswordPolicyResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;
    if payload.min_length < 4 || payload.min_length > 64 {
        return Err(AppError::validation("invalid min_length"));
    }
    let policy = crate::config::PasswordPolicy {
        min_length: payload.min_length,
        require_uppercase: payload.require_uppercase,
        require_lowercase: payload.require_lowercase,
        require_digit: payload.require_digit,
        require_symbol: payload.require_symbol,
    };
    let updated = upsert_password_policy(&state, policy).await?;
    Ok(Json(PasswordPolicyResponse {
        min_length: updated.min_length,
        require_uppercase: updated.require_uppercase,
        require_lowercase: updated.require_lowercase,
        require_digit: updated.require_digit,
        require_symbol: updated.require_symbol,
    }))
}

/// 获取劳动学时规则。
pub async fn get_labor_hour_rules(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<LaborHourRuleRequest>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;
    let rules = load_labor_hour_rules(&state).await?;
    Ok(Json(LaborHourRuleRequest {
        base_hours_a: rules.base_hours_a,
        base_hours_b: rules.base_hours_b,
        national_leader_hours: rules.national_leader_hours,
        national_member_hours: rules.national_member_hours,
        provincial_leader_hours: rules.provincial_leader_hours,
        provincial_member_hours: rules.provincial_member_hours,
        school_leader_hours: rules.school_leader_hours,
        school_member_hours: rules.school_member_hours,
    }))
}

/// 更新劳动学时规则。
pub async fn update_labor_hour_rules(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LaborHourRuleRequest>,
) -> Result<Json<LaborHourRuleRequest>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;
    let updated = upsert_labor_hour_rules(
        &state,
        LaborHourRuleConfig {
            base_hours_a: payload.base_hours_a,
            base_hours_b: payload.base_hours_b,
            national_leader_hours: payload.national_leader_hours,
            national_member_hours: payload.national_member_hours,
            provincial_leader_hours: payload.provincial_leader_hours,
            provincial_member_hours: payload.provincial_member_hours,
            school_leader_hours: payload.school_leader_hours,
            school_member_hours: payload.school_member_hours,
        },
    )
    .await?;
    Ok(Json(LaborHourRuleRequest {
        base_hours_a: updated.base_hours_a,
        base_hours_b: updated.base_hours_b,
        national_leader_hours: updated.national_leader_hours,
        national_member_hours: updated.national_member_hours,
        provincial_leader_hours: updated.provincial_leader_hours,
        provincial_member_hours: updated.provincial_member_hours,
        school_leader_hours: updated.school_leader_hours,
        school_member_hours: updated.school_member_hours,
    }))
}

/// 为用户发送 TOTP 重置链接。
pub async fn reset_user_totp(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<ResetUserRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let admin = require_session_user(&state, &jar).await?;
    require_role(&admin, "admin")?;
    if matches!(state.config.reset_delivery, crate::config::ResetDelivery::Code) {
        return Err(AppError::bad_request("reset delivery set to code"));
    }

    let user = User::find()
        .filter(users::Column::Username.eq(payload.username))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("user not found"))?;
    if user.role == "student" {
        return Err(AppError::bad_request("student reset via email"));
    }
    let email = user.email.ok_or_else(|| AppError::bad_request("email not set"))?;
    let base_url = state
        .config
        .base_url
        .as_ref()
        .ok_or_else(|| AppError::config("BASE_URL is required"))?;
    let mail_config = state
        .config
        .mail
        .as_ref()
        .ok_or_else(|| AppError::config("mail config required"))?;

    let token = generate_token();
    let token_hash = hash_token(&token);
    let now = Utc::now();
    let expires_at = now + ChronoDuration::minutes(RESET_TTL_MINUTES);
    let reset = auth_resets::ActiveModel {
        id: Set(Uuid::new_v4()),
        token_hash: Set(token_hash),
        user_id: Set(user.id),
        purpose: Set("totp".to_string()),
        expires_at: Set(expires_at),
        created_at: Set(now),
        used_at: Set(None),
    };
    auth_resets::Entity::insert(reset)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let link = format!("{}/reset?token={}", base_url, token);
    let body = format!(
        "您好，\n\n请点击以下链接重置您的 TOTP：\n{}\n\n该链接 {} 分钟后失效。",
        link, RESET_TTL_MINUTES
    );
    send_mail(mail_config, &email, "TOTP 重置", &body).await?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// 为用户发送 Passkey 重置链接。
pub async fn reset_user_passkey(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<ResetUserRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let admin = require_session_user(&state, &jar).await?;
    require_role(&admin, "admin")?;
    if matches!(state.config.reset_delivery, crate::config::ResetDelivery::Code) {
        return Err(AppError::bad_request("reset delivery set to code"));
    }

    let user = User::find()
        .filter(users::Column::Username.eq(payload.username))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("user not found"))?;
    if user.role == "student" {
        return Err(AppError::bad_request("student reset via email"));
    }
    let email = user.email.ok_or_else(|| AppError::bad_request("email not set"))?;
    let base_url = state
        .config
        .base_url
        .as_ref()
        .ok_or_else(|| AppError::config("BASE_URL is required"))?;
    let mail_config = state
        .config
        .mail
        .as_ref()
        .ok_or_else(|| AppError::config("mail config required"))?;

    let token = generate_token();
    let token_hash = hash_token(&token);
    let now = Utc::now();
    let expires_at = now + ChronoDuration::minutes(RESET_TTL_MINUTES);
    let reset = auth_resets::ActiveModel {
        id: Set(Uuid::new_v4()),
        token_hash: Set(token_hash),
        user_id: Set(user.id),
        purpose: Set("passkey".to_string()),
        expires_at: Set(expires_at),
        created_at: Set(now),
        used_at: Set(None),
    };
    auth_resets::Entity::insert(reset)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let link = format!("{}/reset?token={}", base_url, token);
    let body = format!(
        "您好，\n\n请点击以下链接重置您的 Passkey：\n{}\n\n该链接 {} 分钟后失效。",
        link, RESET_TTL_MINUTES
    );
    send_mail(mail_config, &email, "Passkey 重置", &body).await?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// 生成一次性重置码（仅内网模式）。
pub async fn generate_reset_code(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<ResetCodeRequest>,
) -> Result<Json<ResetCodeResponse>, AppError> {
    let admin = require_session_user(&state, &jar).await?;
    require_role(&admin, "admin")?;
    if matches!(state.config.reset_delivery, crate::config::ResetDelivery::Email) {
        return Err(AppError::bad_request("reset delivery set to email"));
    }

    let user = User::find()
        .filter(users::Column::Username.eq(&payload.username))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("user not found"))?;

    let purpose = payload.purpose.as_str();
    if purpose == "password" && user.role != "student" {
        return Err(AppError::bad_request("password reset only for students"));
    }
    if (purpose == "totp" || purpose == "passkey") && user.role == "student" {
        return Err(AppError::bad_request("student reset via password"));
    }
    if !matches!(purpose, "password" | "totp" | "passkey") {
        return Err(AppError::validation("invalid reset purpose"));
    }

    let token = generate_token();
    let token_hash = hash_token(&token);
    let now = Utc::now();
    let expires_at = now + ChronoDuration::minutes(RESET_TTL_MINUTES);
    let reset = auth_resets::ActiveModel {
        id: Set(Uuid::new_v4()),
        token_hash: Set(token_hash),
        user_id: Set(user.id),
        purpose: Set(purpose.to_string()),
        expires_at: Set(expires_at),
        created_at: Set(now),
        used_at: Set(None),
    };
    auth_resets::Entity::insert(reset)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(ResetCodeResponse {
        code: Some(token),
        expires_in_minutes: RESET_TTL_MINUTES,
    }))
}

/// 从 Excel 导入竞赛名称（仅管理员）。
pub async fn import_competitions(
    State(state): State<AppState>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let file_bytes = read_upload_bytes(&mut multipart).await?;
    let mut workbook = calamine::Xlsx::new(Cursor::new(file_bytes))
        .map_err(|_| AppError::bad_request("invalid xlsx file"))?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| AppError::bad_request("xlsx has no sheets"))?;
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|_| AppError::bad_request("failed to read worksheet"))?;

    let template = load_import_template(&state, "competition_library").await?;
    let header_index = build_header_index(range.rows().next());
    let field_map = map_import_fields(&header_index, &template.fields)?;

    let mut inserted = 0usize;
    let mut skipped = 0usize;
    for row in range.rows().skip(1) {
        let name = field_map
            .get("contest_name")
            .map(|idx| read_cell_by_index(*idx, row))
            .unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let year = field_map
            .get("contest_year")
            .and_then(|idx| read_cell_by_index(*idx, row).parse::<i32>().ok());
        let category = field_map
            .get("contest_category")
            .map(|idx| read_cell_by_index(*idx, row))
            .filter(|value| !value.is_empty());
        let exists = CompetitionLibrary::find()
            .filter(competition_library::Column::Name.eq(&name))
            .one(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
        if exists.is_some() {
            skipped += 1;
            continue;
        }
        let now = Utc::now();
        let model = competition_library::ActiveModel {
            id: Set(Uuid::new_v4()),
            year: Set(year),
            category: Set(category.map(|value| value.to_uppercase())),
            name: Set(name),
            created_at: Set(now),
            updated_at: Set(now),
        };
        competition_library::Entity::insert(model)
            .exec_without_returning(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
        inserted += 1;
    }

    Ok(Json(serde_json::json!({ "inserted": inserted, "skipped": skipped })))
}

/// 表单字段新增请求。
#[derive(Debug, Deserialize, Validate)]
pub struct CreateFormFieldRequest {
    /// 表单类型。
    #[validate(length(min = 1, max = 32))]
    pub form_type: String,
    /// 字段 key。
    #[validate(length(min = 1, max = 64))]
    pub field_key: String,
    /// 字段标签。
    #[validate(length(min = 1, max = 64))]
    pub label: String,
    /// 字段类型。
    #[validate(length(min = 1, max = 32))]
    pub field_type: String,
    /// 是否必填。
    pub required: bool,
    /// 排序序号。
    pub order_index: i32,
}

/// 表单字段响应。
#[derive(Debug, Serialize)]
pub struct FormFieldResponse {
    /// 记录 ID。
    pub id: Uuid,
    /// 表单类型。
    pub form_type: String,
    /// 字段 key。
    pub field_key: String,
    /// 字段标签。
    pub label: String,
    /// 字段类型。
    pub field_type: String,
    /// 是否必填。
    pub required: bool,
    /// 排序序号。
    pub order_index: i32,
}

/// 查询表单字段。
pub async fn list_form_fields(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<FormFieldResponse>>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let fields = FormField::find()
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(
        fields
            .into_iter()
            .map(|field| FormFieldResponse {
                id: field.id,
                form_type: field.form_type,
                field_key: field.field_key,
                label: field.label,
                field_type: field.field_type,
                required: field.required,
                order_index: field.order_index,
            })
            .collect(),
    ))
}

/// 新增表单字段。
pub async fn create_form_field(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<CreateFormFieldRequest>,
) -> Result<Json<FormFieldResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;
    payload
        .validate()
        .map_err(|_| AppError::validation("invalid form field payload"))?;

    let now = Utc::now();
    let id = Uuid::new_v4();
    let model = form_fields::ActiveModel {
        id: Set(id),
        form_type: Set(payload.form_type.clone()),
        field_key: Set(payload.field_key.clone()),
        label: Set(payload.label.clone()),
        field_type: Set(payload.field_type.clone()),
        required: Set(payload.required),
        order_index: Set(payload.order_index),
        created_at: Set(now),
        updated_at: Set(now),
    };
    form_fields::Entity::insert(model)
        .exec_without_returning(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(FormFieldResponse {
        id,
        form_type: payload.form_type,
        field_key: payload.field_key,
        label: payload.label,
        field_type: payload.field_type,
        required: payload.required,
        order_index: payload.order_index,
    }))
}

/// 获取导入模板列表（仅管理员）。
pub async fn list_import_templates(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<ImportTemplateResponse>>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let mut templates = Vec::new();
    for key in IMPORT_TEMPLATE_KEYS {
        let config = load_import_template(&state, key).await?;
        templates.push(import_template_to_response(config));
    }
    Ok(Json(templates))
}

/// 更新导入模板（仅管理员）。
pub async fn update_import_template(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(template_key): Path<String>,
    Json(payload): Json<ImportTemplateRequest>,
) -> Result<Json<ImportTemplateResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;
    payload
        .validate()
        .map_err(|_| AppError::validation("invalid import template payload"))?;

    if !IMPORT_TEMPLATE_KEYS.contains(&template_key.as_str()) {
        return Err(AppError::bad_request("unknown template key"));
    }

    let mut seen = HashSet::new();
    for field in &payload.fields {
        field
            .validate()
            .map_err(|_| AppError::validation("invalid import template field"))?;
        if field.required && field.column_title.trim().is_empty() {
            return Err(AppError::validation("required column title missing"));
        }
        if !seen.insert(field.field_key.as_str()) {
            return Err(AppError::validation("duplicate field key"));
        }
    }

    let transaction = state
        .db
        .begin()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let now = Utc::now();
    let template = ImportTemplate::find()
        .filter(import_templates::Column::TemplateKey.eq(&template_key))
        .one(&transaction)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    let template_id = if let Some(existing) = template {
        let mut active: import_templates::ActiveModel = existing.into();
        active.name = Set(payload.name.clone());
        active.updated_at = Set(now);
        let updated = active
            .update(&transaction)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
        updated.id
    } else {
        let model = import_templates::ActiveModel {
            id: Set(Uuid::new_v4()),
            template_key: Set(template_key.clone()),
            name: Set(payload.name.clone()),
            created_at: Set(now),
            updated_at: Set(now),
        };
        import_templates::Entity::insert(model)
            .exec(&transaction)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?
            .last_insert_id
    };

    ImportTemplateField::delete_many()
        .filter(import_template_fields::Column::TemplateId.eq(template_id))
        .exec(&transaction)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    for field in &payload.fields {
        let model = import_template_fields::ActiveModel {
            id: Set(Uuid::new_v4()),
            template_id: Set(template_id),
            field_key: Set(field.field_key.clone()),
            label: Set(field.label.clone()),
            column_title: Set(field.column_title.clone()),
            required: Set(field.required),
            order_index: Set(field.order_index),
            description: Set(field.description.clone()),
            created_at: Set(now),
            updated_at: Set(now),
        };
        import_template_fields::Entity::insert(model)
            .exec_without_returning(&transaction)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    }

    transaction
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let config = ImportTemplateConfig {
        template_key,
        name: payload.name,
        fields: payload
            .fields
            .into_iter()
            .map(|field| ImportFieldConfig {
                field_key: field.field_key,
                label: field.label,
                column_title: field.column_title,
                required: field.required,
                order_index: field.order_index,
                description: field.description,
            })
            .collect(),
    };
    Ok(Json(import_template_to_response(config)))
}

/// 获取导出模板（仅管理员）。
pub async fn get_export_template(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(template_key): Path<String>,
) -> Result<Json<ExportTemplateResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;
    if !EXPORT_TEMPLATE_KEYS.contains(&template_key.as_str()) {
        return Err(AppError::bad_request("unknown template key"));
    }

    let config = load_export_template(&state, &template_key).await?;
    Ok(Json(export_template_to_response(config)))
}

/// 更新导出模板（仅管理员）。
pub async fn update_export_template(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(template_key): Path<String>,
    Json(payload): Json<ExportTemplateRequest>,
) -> Result<Json<ExportTemplateResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;
    payload
        .validate()
        .map_err(|_| AppError::validation("invalid export template payload"))?;
    if !EXPORT_TEMPLATE_KEYS.contains(&template_key.as_str()) {
        return Err(AppError::bad_request("unknown template key"));
    }

    let updated =
        upsert_export_template(&state, &template_key, payload.name, payload.layout).await?;
    Ok(Json(export_template_to_response(updated)))
}

/// 已删除竞赛记录响应。
#[derive(Debug, Serialize)]
pub struct DeletedContestRecordResponse {
    /// 记录 ID。
    pub id: Uuid,
    /// 学生 ID。
    pub student_id: Uuid,
    /// 竞赛名称。
    pub contest_name: String,
    /// 状态。
    pub status: String,
    /// 创建时间。
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 获取已删除学生列表（仅管理员）。
pub async fn list_deleted_students(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<crate::routes::students::StudentResponse>>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let results = Student::find()
        .filter(students::Column::IsDeleted.eq(true))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(
        results
            .into_iter()
            .map(crate::routes::students::StudentResponse::from)
            .collect(),
    ))
}

/// 获取已删除竞赛记录（仅管理员）。
pub async fn list_deleted_contest_records(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<DeletedContestRecordResponse>>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let records = ContestRecord::find()
        .filter(contest_records::Column::IsDeleted.eq(true))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(
        records
            .into_iter()
            .map(|record| DeletedContestRecordResponse {
                id: record.id,
                student_id: record.student_id,
                contest_name: record.contest_name,
                status: record.status,
                created_at: record.created_at,
            })
            .collect(),
    ))
}

/// 删除学生（仅管理员，软删除）。
pub async fn delete_student(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(student_no): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let student = Student::find()
        .filter(students::Column::StudentNo.eq(&student_no))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("student not found"))?;

    if student.is_deleted {
        return Ok(Json(serde_json::json!({ "deleted": true })));
    }

    let mut active: students::ActiveModel = student.into();
    active.is_deleted = Set(true);
    active.updated_at = Set(Utc::now());
    active
        .update(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

/// 彻底删除学生（仅管理员）。
pub async fn purge_student(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(student_no): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let student = Student::find()
        .filter(students::Column::StudentNo.eq(&student_no))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("student not found"))?;

    if !student.is_deleted {
        return Err(AppError::bad_request("student must be soft deleted first"));
    }

    let transaction = state
        .db
        .begin()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let contest_records = ContestRecord::find()
        .filter(contest_records::Column::StudentId.eq(student.id))
        .all(&transaction)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    let contest_ids: Vec<Uuid> = contest_records.iter().map(|record| record.id).collect();

    if !contest_ids.is_empty() {
        FormFieldValue::delete_many()
            .filter(form_field_values::Column::RecordType.eq("contest"))
            .filter(form_field_values::Column::RecordId.is_in(contest_ids.iter().cloned()))
            .exec(&transaction)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
        ReviewSignature::delete_many()
            .filter(review_signatures::Column::RecordType.eq("contest"))
            .filter(review_signatures::Column::RecordId.is_in(contest_ids.iter().cloned()))
            .exec(&transaction)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    }

    Attachment::delete_many()
        .filter(attachments::Column::StudentId.eq(student.id))
        .exec(&transaction)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    ContestRecord::delete_many()
        .filter(contest_records::Column::StudentId.eq(student.id))
        .exec(&transaction)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Student::delete_by_id(student.id)
        .exec(&transaction)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

/// 删除未审核竞赛记录（仅管理员，软删除）。
pub async fn delete_contest_record(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(record_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let record = ContestRecord::find()
        .filter(contest_records::Column::Id.eq(record_id))
        .filter(contest_records::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("record not found"))?;

    if record.status != "submitted" {
        return Err(AppError::bad_request("reviewed record cannot be deleted"));
    }

    let mut active: contest_records::ActiveModel = record.into();
    active.is_deleted = Set(true);
    active.updated_at = Set(Utc::now());
    active
        .update(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

/// 彻底删除竞赛记录（仅管理员）。
pub async fn purge_contest_record(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(record_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let record = ContestRecord::find()
        .filter(contest_records::Column::Id.eq(record_id))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("record not found"))?;
    if !record.is_deleted {
        return Err(AppError::bad_request("record must be soft deleted first"));
    }

    let transaction = state
        .db
        .begin()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    FormFieldValue::delete_many()
        .filter(form_field_values::Column::RecordType.eq("contest"))
        .filter(form_field_values::Column::RecordId.eq(record_id))
        .exec(&transaction)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    ReviewSignature::delete_many()
        .filter(review_signatures::Column::RecordType.eq("contest"))
        .filter(review_signatures::Column::RecordId.eq(record_id))
        .exec(&transaction)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Attachment::delete_many()
        .filter(attachments::Column::RecordType.eq("contest"))
        .filter(attachments::Column::RecordId.eq(record_id))
        .exec(&transaction)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    ContestRecord::delete_by_id(record_id)
        .exec(&transaction)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

/// 批量导入竞赛记录（仅管理员）。
pub async fn import_contest_records(
    State(state): State<AppState>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let file_bytes = read_upload_bytes(&mut multipart).await?;
    let mut workbook = calamine::Xlsx::new(Cursor::new(file_bytes))
        .map_err(|_| AppError::bad_request("invalid xlsx file"))?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| AppError::bad_request("xlsx has no sheets"))?;
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|_| AppError::bad_request("failed to read worksheet"))?;

    let template = load_import_template(&state, "contest_records").await?;
    let header_index = build_header_index(range.rows().next());
    let base_index = map_import_fields(&header_index, &template.fields)?;

    let field_map = load_form_field_map(&state, "contest").await?;

    let transaction = state
        .db
        .begin()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut inserted = 0usize;
    let mut skipped = 0usize;
    for row in range.rows().skip(1) {
        let student_no = read_cell_by_index_opt(base_index.get("student_no"), row);
        if student_no.is_empty() {
            skipped += 1;
            continue;
        }

        let student = Student::find()
            .filter(students::Column::StudentNo.eq(&student_no))
            .filter(students::Column::IsDeleted.eq(false))
            .one(&transaction)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
        let student = match student {
            Some(student) => student,
            None => {
                skipped += 1;
                continue;
            }
        };

        let contest_name = read_cell_by_index_opt(base_index.get("contest_name"), row);
        let contest_level = read_cell_by_index_opt(base_index.get("contest_level"), row);
        let contest_role = read_cell_by_index_opt(base_index.get("contest_role"), row);
        let award_level = read_cell_by_index_opt(base_index.get("award_level"), row);
        let self_hours = parse_hours(read_cell_by_index_opt(base_index.get("self_hours"), row));
        let contest_year = read_cell_by_index_opt(base_index.get("contest_year"), row)
            .parse::<i32>()
            .ok();
        let contest_category = read_cell_by_index_opt(base_index.get("contest_category"), row);
        let award_date = read_cell_by_index_opt(base_index.get("award_date"), row);
        if contest_name.is_empty()
            || contest_level.is_empty()
            || contest_role.is_empty()
            || award_level.is_empty()
            || self_hours.is_none()
        {
            skipped += 1;
            continue;
        }

        let first_review = parse_hours(read_cell_by_index_opt(base_index.get("first_review_hours"), row));
        let final_review = parse_hours(read_cell_by_index_opt(base_index.get("final_review_hours"), row));
        let status_value = read_cell_by_index_opt(base_index.get("status"), row);
        let rejection = read_cell_by_index_opt(base_index.get("rejection_reason"), row);
        let status = resolve_status(&status_value, first_review, final_review);

        let now = Utc::now();
        let award_date = parse_award_date_cell(&award_date)?;
        let record_id = Uuid::new_v4();
        let model = contest_records::ActiveModel {
            id: Set(record_id),
            student_id: Set(student.id),
            contest_year: Set(contest_year),
            contest_category: Set(if contest_category.is_empty() { None } else { Some(contest_category.to_uppercase()) }),
            contest_name: Set(contest_name),
            contest_level: Set(Some(contest_level)),
            contest_role: Set(Some(contest_role)),
            award_level: Set(award_level),
            award_date: Set(award_date),
            self_hours: Set(self_hours.unwrap_or(0)),
            first_review_hours: Set(first_review),
            final_review_hours: Set(final_review),
            status: Set(status),
            rejection_reason: Set(if rejection.is_empty() { None } else { Some(rejection) }),
            is_deleted: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
        };
        contest_records::Entity::insert(model)
            .exec_without_returning(&transaction)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;

        let reserved_headers = template
            .fields
            .iter()
            .map(|field| field.column_title.clone())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        insert_custom_fields(
            &transaction,
            "contest",
            record_id,
            row,
            &header_index,
            &field_map,
            &reserved_headers,
        )
        .await?;
        inserted += 1;
    }

    transaction
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(serde_json::json!({ "inserted": inserted, "skipped": skipped })))
}

async fn read_upload_bytes(multipart: &mut Multipart) -> Result<Vec<u8>, AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::bad_request("invalid multipart"))?
    {
        if field.name() == Some("file") {
            let bytes = field
                .bytes()
                .await
                .map_err(|_| AppError::bad_request("failed to read file"))?;
            return Ok(bytes.to_vec());
        }
    }
    Err(AppError::bad_request("file field required"))
}

fn build_header_index(header_row: Option<&[Data]>) -> HashMap<String, usize> {
    let mut header_index = HashMap::new();
    if let Some(header_row) = header_row {
        for (idx, cell) in header_row.iter().enumerate() {
            let trimmed = cell.to_string().trim().to_string();
            if !trimmed.is_empty() {
                header_index.insert(trimmed, idx);
            }
        }
    }
    header_index
}

fn find_header_index(
    header_index: &HashMap<String, usize>,
    candidates: &[&str],
) -> Option<usize> {
    candidates.iter().find_map(|key| header_index.get(*key).cloned())
}

fn map_base_indices(
    header_index: &HashMap<String, usize>,
    candidates: &[(&str, &[&str])],
) -> HashMap<String, usize> {
    let mut result = HashMap::new();
    for (key, headers) in candidates {
        if let Some(idx) = find_header_index(header_index, headers) {
            result.insert(key.to_string(), idx);
        }
    }
    result
}

fn import_template_to_response(template: ImportTemplateConfig) -> ImportTemplateResponse {
    ImportTemplateResponse {
        template_key: template.template_key,
        name: template.name,
        fields: template
            .fields
            .into_iter()
            .map(|field| ImportTemplateFieldResponse {
                field_key: field.field_key,
                label: field.label,
                column_title: field.column_title,
                required: field.required,
                order_index: field.order_index,
                description: field.description,
            })
            .collect(),
    }
}

fn export_template_to_response(template: ExportTemplateConfig) -> ExportTemplateResponse {
    ExportTemplateResponse {
        template_key: template.template_key,
        name: template.name,
        layout: template.layout,
    }
}

fn ensure_required_headers(
    base_index: &HashMap<String, usize>,
    required: &[&str],
) -> Result<(), AppError> {
    for key in required {
        if !base_index.contains_key(*key) {
            return Err(AppError::bad_request("missing required header"));
        }
    }
    Ok(())
}

fn read_cell_by_index_opt(index: Option<&usize>, row: &[Data]) -> String {
    let idx = match index {
        Some(value) => *value,
        None => return String::new(),
    };
    read_cell_by_index(idx, row)
}

fn read_cell_by_index(idx: usize, row: &[Data]) -> String {
    row.get(idx)
        .map(|cell| cell.to_string().trim().to_string())
        .unwrap_or_default()
}

fn parse_award_date_cell(value: &str) -> Result<Option<chrono::DateTime<Utc>>, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return Ok(Some(dt.with_timezone(&Utc)));
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        let dt = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| AppError::validation("invalid award date"))?;
        return Ok(Some(chrono::DateTime::<Utc>::from_utc(dt, Utc)));
    }
    Err(AppError::validation("invalid award date"))
}

fn parse_hours(value: String) -> Option<i32> {
    if value.is_empty() {
        return None;
    }
    value.parse::<f32>().ok().map(|num| num.round() as i32)
}

fn resolve_status(status_value: &str, first_review: Option<i32>, final_review: Option<i32>) -> String {
    if status_value == "不通过" || status_value == "rejected" {
        return "rejected".to_string();
    }
    if status_value == "已复审" || status_value == "final_reviewed" || final_review.is_some() {
        return "final_reviewed".to_string();
    }
    if status_value == "已初审" || status_value == "first_reviewed" || first_review.is_some() {
        return "first_reviewed".to_string();
    }
    "submitted".to_string()
}

async fn load_form_field_map(
    state: &AppState,
    form_type: &str,
) -> Result<HashMap<String, form_fields::Model>, AppError> {
    let fields = FormField::find()
        .filter(form_fields::Column::FormType.eq(form_type))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut map = HashMap::new();
    for field in fields {
        map.insert(field.field_key.clone(), field.clone());
        map.insert(field.label.clone(), field);
    }
    Ok(map)
}

fn collect_reserved_headers(
    header_index: &HashMap<String, usize>,
    base_headers: &[(&str, &[&str])],
    status_headers: &[&str],
    rejection_headers: &[&str],
) -> Vec<String> {
    let mut reserved = Vec::new();
    for (_, headers) in base_headers {
        for header in *headers {
            if header_index.contains_key(*header) {
                reserved.push((*header).to_string());
            }
        }
    }
    for header in status_headers {
        if header_index.contains_key(*header) {
            reserved.push((*header).to_string());
        }
    }
    for header in rejection_headers {
        if header_index.contains_key(*header) {
            reserved.push((*header).to_string());
        }
    }
    reserved
}

async fn insert_custom_fields(
    txn: &sea_orm::DatabaseTransaction,
    record_type: &str,
    record_id: Uuid,
    row: &[Data],
    header_index: &HashMap<String, usize>,
    field_map: &HashMap<String, form_fields::Model>,
    reserved_headers: &[String],
) -> Result<(), AppError> {
    for (header, idx) in header_index {
        if reserved_headers.contains(header) {
            continue;
        }
        if let Some(field) = field_map.get(header) {
            let value = read_cell_by_index(*idx, row);
            if value.is_empty() {
                continue;
            }
            let value_model = form_field_values::ActiveModel {
                id: Set(Uuid::new_v4()),
                record_type: Set(record_type.to_string()),
                record_id: Set(record_id),
                field_key: Set(field.field_key.clone()),
                value: Set(value),
                created_at: Set(Utc::now()),
            };
            form_field_values::Entity::insert(value_model)
                .exec_without_returning(txn)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::Data;

    #[test]
    fn build_header_index_trims_and_ignores_empty() {
        let row = vec![
            Data::String(" 学号 ".to_string()),
            Data::String("".to_string()),
            Data::String("姓名".to_string()),
        ];
        let index = build_header_index(Some(&row));
        assert_eq!(index.get("学号").copied(), Some(0));
        assert_eq!(index.get("姓名").copied(), Some(2));
        assert!(!index.contains_key(""));
    }

    #[test]
    fn map_base_indices_resolves_candidates() {
        let mut header_index = HashMap::new();
        header_index.insert("学号".to_string(), 0);
        header_index.insert("标题".to_string(), 1);
        let result = map_base_indices(&header_index, &[("student_no", &["学号"]), ("title", &["标题"])]);
        assert_eq!(result.get("student_no").copied(), Some(0));
        assert_eq!(result.get("title").copied(), Some(1));
    }

    #[test]
    fn ensure_required_headers_detects_missing() {
        let mut base = HashMap::new();
        base.insert("student_no".to_string(), 0);
        let result = ensure_required_headers(&base, &["student_no", "title"]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_hours_handles_rounding() {
        assert_eq!(parse_hours("1.6".to_string()), Some(2));
        assert_eq!(parse_hours("2".to_string()), Some(2));
        assert_eq!(parse_hours("".to_string()), None);
    }

    #[test]
    fn resolve_status_prefers_rejection_then_reviews() {
        assert_eq!(resolve_status("不通过", None, None), "rejected");
        assert_eq!(resolve_status("已复审", None, None), "final_reviewed");
        assert_eq!(resolve_status("", Some(1), None), "first_reviewed");
        assert_eq!(resolve_status("", None, None), "submitted");
    }

    #[test]
    fn collect_reserved_headers_includes_known() {
        let mut index = HashMap::new();
        index.insert("学号".to_string(), 0);
        index.insert("审核状态".to_string(), 1);
        index.insert("备注".to_string(), 2);
        let reserved = collect_reserved_headers(
            &index,
            &[("student_no", &["学号"])],
            &["审核状态"],
            &["备注"],
        );
        assert!(reserved.contains(&"学号".to_string()));
        assert!(reserved.contains(&"审核状态".to_string()));
        assert!(reserved.contains(&"备注".to_string()));
    }
}
