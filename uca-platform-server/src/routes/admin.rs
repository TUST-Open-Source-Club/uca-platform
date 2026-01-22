//! 管理员维护接口。

use axum::{extract::{State, Multipart, Path}, Json};
use axum_extra::extract::cookie::CookieJar;
use calamine::{Data, Reader};
use chrono::{Duration as ChronoDuration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use uuid::Uuid;
use validator::Validate;

use crate::{
    access::{require_role, require_session_user},
    auth::{generate_token, hash_password, hash_token},
    entities::{
        attachments, auth_resets, competition_library, contest_records, form_field_values, form_fields,
        invites, review_signatures, students, users,
        Attachment, CompetitionLibrary, ContestRecord, FormField, FormFieldValue,
        ReviewSignature, Student, User,
    },
    error::AppError,
    labor_hours::{load_labor_hour_rules, upsert_labor_hour_rules, LaborHourRuleConfig},
    mailer::send_mail,
    policy::{load_password_policy, upsert_password_policy},
    state::AppState,
    templates::{
        export_template_file_path, load_export_template, upsert_export_template_meta,
        ExportTemplateConfig,
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

/// 竞赛库更新请求。
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCompetitionRequest {
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
#[derive(Debug, Deserialize, Serialize)]
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
    /// 内网模式下的认证重置用途（totp/passkey）。
    pub reset_purpose: Option<String>,
}

/// 新建用户响应。
#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    /// 用户 ID。
    pub user_id: Option<Uuid>,
    /// 是否已发送邀请邮件。
    pub invite_sent: bool,
    /// 内网模式下生成的重置码。
    pub reset_code: Option<String>,
    /// 重置用途（totp/passkey）。
    pub reset_purpose: Option<String>,
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

/// 导出模板响应（文件信息）。
#[derive(Debug, Serialize)]
pub struct ExportTemplateResponse {
    pub template_key: String,
    pub name: String,
    pub issues: Vec<String>,
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

#[derive(Debug, Deserialize)]
struct CompetitionSheetPlan {
    name: String,
    year: Option<i32>,
    name_column: Option<String>,
    category_column: Option<String>,
    category_suffix: Option<String>,
}

const INVITE_TTL_HOURS: i64 = 72;
const RESET_TTL_MINUTES: i64 = 24 * 60;

const COMPETITION_HEADER: [&str; 2] = ["竞赛名称", "name"];
const COMPETITION_CATEGORY_HEADERS: [&str; 3] = ["竞赛类型", "竞赛类别", "category"];
const COMPETITION_YEAR_HEADERS: [&str; 3] = ["年份", "year", "年度"];
const CONTEST_IMPORT_HEADERS: [(&str, &[&str]); 13] = [
    ("student_no", &["学号", "student_no"]),
    ("contest_name", &["竞赛名称", "contest_name"]),
    ("contest_level", &["竞赛级别", "contest_level"]),
    ("contest_role", &["角色", "contest_role"]),
    ("award_level", &["获奖等级", "award_level"]),
    ("self_hours", &["自评学时", "self_hours"]),
    ("contest_year", &["年份", "contest_year", "年度"]),
    ("contest_category", &["竞赛类型", "竞赛类别", "contest_category"]),
    ("award_date", &["时间", "award_date", "获奖时间"]),
    ("first_review_hours", &["初审学时", "first_review_hours"]),
    ("final_review_hours", &["复审学时", "final_review_hours"]),
    ("status", &["审核状态", "status"]),
    ("rejection_reason", &["不通过原因", "rejection_reason"]),
];
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
    let category = payload.category.as_ref().map(|value| value.to_uppercase());
    let model = competition_library::ActiveModel {
        id: Set(id),
        year: Set(payload.year),
        category: Set(category.clone()),
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
        category,
        name,
    }))
}

/// 更新竞赛名称库记录。
pub async fn update_competition(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(competition_id): Path<Uuid>,
    Json(payload): Json<UpdateCompetitionRequest>,
) -> Result<Json<CompetitionResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;
    payload
        .validate()
        .map_err(|_| AppError::validation("invalid competition payload"))?;

    let existing = CompetitionLibrary::find()
        .filter(competition_library::Column::Id.eq(competition_id))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("competition not found"))?;

    let name_exists = CompetitionLibrary::find()
        .filter(competition_library::Column::Name.eq(&payload.name))
        .filter(competition_library::Column::Id.ne(competition_id))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    if name_exists.is_some() {
        return Err(AppError::bad_request("competition exists"));
    }

    let mut active: competition_library::ActiveModel = existing.into();
    let category = payload.category.as_ref().map(|value| value.to_uppercase());
    active.name = Set(payload.name.clone());
    active.year = Set(payload.year);
    active.category = Set(category.clone());
    active.updated_at = Set(Utc::now());
    let model = active
        .update(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(CompetitionResponse {
        id: model.id,
        year: model.year,
        category: model.category,
        name: model.name,
    }))
}

/// 删除竞赛名称库记录。
pub async fn delete_competition(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(competition_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let result = CompetitionLibrary::delete_by_id(competition_id)
        .exec(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    if result.rows_affected == 0 {
        return Err(AppError::not_found("competition not found"));
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
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
            active.must_change_password = Set(true);
            active.updated_at = Set(now);
            let model = active
                .update(&state.db)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?;
            return Ok(Json(CreateUserResponse {
                user_id: Some(model.id),
                invite_sent: false,
                reset_code: None,
                reset_purpose: None,
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
            must_change_password: Set(true),
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
            reset_code: None,
            reset_purpose: None,
        }));
    }

    let existing = User::find()
        .filter(users::Column::Username.eq(&payload.username))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    if existing.is_some() {
        return Err(AppError::bad_request("user already exists"));
    }

    if matches!(state.config.reset_delivery, crate::config::ResetDelivery::Code) {
        let now = Utc::now();
        let user_id = Uuid::new_v4();
        let model = users::ActiveModel {
            id: Set(user_id),
            username: Set(payload.username.clone()),
            display_name: Set(payload.display_name.clone()),
            role: Set(payload.role.clone()),
            email: Set(payload.email.clone()),
            password_hash: Set(None),
            allow_password_login: Set(false),
            password_updated_at: Set(None),
            must_change_password: Set(false),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
        };
        users::Entity::insert(model)
            .exec_without_returning(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;

        let purpose = payload
            .reset_purpose
            .clone()
            .unwrap_or_else(|| "totp".to_string());
        if !matches!(purpose.as_str(), "totp" | "passkey") {
            return Err(AppError::validation("invalid reset purpose"));
        }

        let token = generate_token();
        let token_hash = hash_token(&token);
        let expires_at = now + ChronoDuration::minutes(RESET_TTL_MINUTES);
        let reset = auth_resets::ActiveModel {
            id: Set(Uuid::new_v4()),
            token_hash: Set(token_hash),
            user_id: Set(user_id),
            purpose: Set(purpose.clone()),
            expires_at: Set(expires_at),
            created_at: Set(now),
            used_at: Set(None),
        };
        auth_resets::Entity::insert(reset)
            .exec_without_returning(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;

        return Ok(Json(CreateUserResponse {
            user_id: Some(user_id),
            invite_sent: false,
            reset_code: Some(token),
            reset_purpose: Some(purpose),
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
        reset_code: None,
        reset_purpose: None,
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
        "您好，\n\n请点击以下链接重置您的 TOTP：\n{}\n\n该链接 {} 小时后失效。",
        link,
        RESET_TTL_MINUTES / 60
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
        "您好，\n\n请点击以下链接重置您的 Passkey：\n{}\n\n该链接 {} 小时后失效。",
        link,
        RESET_TTL_MINUTES / 60
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

    let (file_bytes, fields) = read_upload_payload(&mut multipart).await?;
    let default_year = fields
        .get("default_year")
        .and_then(|value| value.parse::<i32>().ok());
    let sheet_plan = fields
        .get("sheet_plan")
        .map(|value| serde_json::from_str::<Vec<CompetitionSheetPlan>>(value))
        .transpose()
        .map_err(|_| AppError::bad_request("invalid sheet_plan"))?;
    let mut workbook = calamine::Xlsx::new(Cursor::new(file_bytes))
        .map_err(|_| AppError::bad_request("invalid xlsx file"))?;
    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err(AppError::bad_request("xlsx has no sheets"));
    }

    let mut inserted = 0usize;
    let mut skipped = 0usize;

    let plans = if let Some(plan) = sheet_plan {
        if plan.is_empty() {
            return Err(AppError::bad_request("sheet_plan empty"));
        }
        plan
    } else {
        vec![CompetitionSheetPlan {
            name: sheet_names[0].clone(),
            year: None,
            name_column: None,
            category_column: None,
            category_suffix: None,
        }]
    };

    for plan in plans {
        if !sheet_names.contains(&plan.name) {
            let message = format!("worksheet not found: {}", plan.name);
            return Err(AppError::bad_request(&message));
        }
        if let Some(suffix) = plan.category_suffix.as_deref() {
            if !matches!(suffix, "class" | "class_contest") {
                return Err(AppError::validation("invalid category_suffix"));
            }
        }
        let range = workbook
            .worksheet_range(&plan.name)
            .map_err(|_| AppError::bad_request("failed to read worksheet"))?;

        let header_index = build_header_index(range.rows().next());
        let name_idx = resolve_column_index(
            &header_index,
            plan.name_column.as_deref(),
            &COMPETITION_HEADER,
        )
        .ok_or_else(|| AppError::bad_request("missing contest name column"))?;
        let category_idx = resolve_column_index(
            &header_index,
            plan.category_column.as_deref(),
            &COMPETITION_CATEGORY_HEADERS,
        );
        let year_idx = resolve_column_index(&header_index, None, &COMPETITION_YEAR_HEADERS);
        let sheet_default_year = plan.year.or(default_year);
        if year_idx.is_none() && sheet_default_year.is_none() {
            let message = format!(
                "default_year required when year column missing in {}",
                plan.name
            );
            return Err(AppError::bad_request(&message));
        }

        for row in range.rows().skip(1) {
            let name = read_cell_by_index(name_idx, row);
            if name.is_empty() {
                continue;
            }
            let year = year_idx
                .and_then(|idx| read_cell_by_index(idx, row).parse::<i32>().ok())
                .or(sheet_default_year);
            let category = read_cell_by_index_opt(category_idx.as_ref(), row)
                .trim()
                .to_string();
            let category = if category.is_empty() {
                None
            } else {
                Some(normalize_category(&category, plan.category_suffix.as_deref()))
            };
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

    let mut config = load_export_template(&state, &template_key).await?;
    let template_path = export_template_file_path(&state, &template_key);
    if !template_path.exists() {
        config.name.clear();
        config.issues.clear();
    }
    Ok(Json(export_template_to_response(config)))
}

/// 上传导出模板（仅管理员）。
pub async fn upload_export_template(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(template_key): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<ExportTemplateResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;
    if !EXPORT_TEMPLATE_KEYS.contains(&template_key.as_str()) {
        return Err(AppError::bad_request("unknown template key"));
    }

    let (file_bytes, file_name) = read_upload_file(&mut multipart).await?;
    let issues = crate::export_template::validate_export_template_bytes(&file_bytes)?;

    let template_path = export_template_file_path(&state, &template_key);
    if let Some(parent) = template_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| AppError::internal(&format!("create template dir failed: {err}")))?;
    }
    std::fs::write(&template_path, &file_bytes)
        .map_err(|err| AppError::internal(&format!("save template failed: {err}")))?;

    let updated =
        upsert_export_template_meta(&state, &template_key, file_name, issues).await?;
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

/// 更新学生登录权限请求。
#[derive(Debug, Deserialize)]
pub struct UpdateStudentLoginRequest {
    /// 是否允许学生使用密码登录。
    pub allow_login: bool,
}

/// 学生用户密码规则。
#[derive(Debug, Deserialize)]
pub struct StudentPasswordRule {
    /// 固定前缀。
    pub prefix: Option<String>,
    /// 固定后缀。
    pub suffix: Option<String>,
    /// 是否包含学号。
    pub include_student_no: bool,
    /// 是否包含手机号。
    pub include_phone: bool,
}

/// 批量为学生创建用户请求。
#[derive(Debug, Deserialize)]
pub struct CreateStudentUsersRequest {
    /// 学号列表。
    pub student_nos: Vec<String>,
    /// 密码生成规则。
    pub password_rule: StudentPasswordRule,
}

/// 批量创建学生用户响应。
#[derive(Debug, Serialize)]
pub struct CreateStudentUsersResponse {
    /// 成功创建数量。
    pub created: usize,
    /// 已存在跳过数量。
    pub skipped: usize,
    /// 生成的密码清单。
    pub passwords: Vec<GeneratedStudentPassword>,
}

/// 生成的学生密码条目。
#[derive(Debug, Serialize)]
pub struct GeneratedStudentPassword {
    /// 学号。
    pub student_no: String,
    /// 生成的密码。
    pub password: String,
}

fn build_student_password(
    rule: &StudentPasswordRule,
    student: &students::Model,
) -> Result<String, AppError> {
    let mut parts: Vec<String> = Vec::new();
    if let Some(prefix) = rule.prefix.as_ref() {
        if !prefix.is_empty() {
            parts.push(prefix.clone());
        }
    }
    if rule.include_student_no {
        parts.push(student.student_no.clone());
    }
    if rule.include_phone {
        if student.phone.is_empty() {
            return Err(AppError::bad_request("student phone missing"));
        }
        parts.push(student.phone.clone());
    }
    if let Some(suffix) = rule.suffix.as_ref() {
        if !suffix.is_empty() {
            parts.push(suffix.clone());
        }
    }
    let password = parts.join("");
    if password.is_empty() {
        return Err(AppError::bad_request("password rule produces empty password"));
    }
    Ok(password)
}

/// 修改学生是否允许密码登录（仅管理员）。
pub async fn update_student_login(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(student_no): Path<String>,
    Json(payload): Json<UpdateStudentLoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let student = Student::find()
        .filter(students::Column::StudentNo.eq(&student_no))
        .filter(students::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("student not found"))?;

    let now = Utc::now();
    if let Some(existing) = User::find()
        .filter(users::Column::Username.eq(&student.student_no))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
    {
        let mut active: users::ActiveModel = existing.into();
        active.allow_password_login = Set(payload.allow_login);
        if payload.allow_login {
            active.must_change_password = Set(true);
        }
        active.updated_at = Set(now);
        active
            .update(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    } else {
        let default_password = format!("st{}", student.student_no);
        let default_hash = hash_password(&default_password)?;
        let model = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            username: Set(student.student_no.clone()),
            display_name: Set(student.name.clone()),
            role: Set("student".to_string()),
            email: Set(None),
            password_hash: Set(Some(default_hash)),
            allow_password_login: Set(payload.allow_login),
            password_updated_at: Set(Some(now)),
            must_change_password: Set(payload.allow_login),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
        };
        users::Entity::insert(model)
            .exec_without_returning(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

/// 批量为学生创建用户（仅管理员）。
pub async fn create_student_users(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<CreateStudentUsersRequest>,
) -> Result<Json<CreateStudentUsersResponse>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    if payload.student_nos.is_empty() {
        return Err(AppError::bad_request("student_nos required"));
    }

    let students_list = Student::find()
        .filter(students::Column::StudentNo.is_in(payload.student_nos.clone()))
        .filter(students::Column::IsDeleted.eq(false))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut created = 0usize;
    let mut skipped = 0usize;
    let mut passwords = Vec::new();
    let now = Utc::now();

    for student in students_list {
        let exists = User::find()
            .filter(users::Column::Username.eq(&student.student_no))
            .one(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
        if exists.is_some() {
            skipped += 1;
            continue;
        }
        let password = build_student_password(&payload.password_rule, &student)?;
        let hash = hash_password(&password)?;
        let user_id = Uuid::new_v4();
        let model = users::ActiveModel {
            id: Set(user_id),
            username: Set(student.student_no.clone()),
            display_name: Set(student.name.clone()),
            role: Set("student".to_string()),
            email: Set(None),
            password_hash: Set(Some(hash)),
            allow_password_login: Set(true),
            password_updated_at: Set(Some(now)),
            must_change_password: Set(true),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
        };
        users::Entity::insert(model)
            .exec_without_returning(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
        passwords.push(GeneratedStudentPassword {
            student_no: student.student_no,
            password,
        });
        created += 1;
    }

    Ok(Json(CreateStudentUsersResponse {
        created,
        skipped,
        passwords,
    }))
}

/// 重置学生默认密码（仅管理员）。
pub async fn reset_student_password(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(student_no): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = require_session_user(&state, &jar).await?;
    require_role(&user, "admin")?;

    let student = Student::find()
        .filter(students::Column::StudentNo.eq(&student_no))
        .filter(students::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("student not found"))?;

    let default_password = format!("st{}", student.student_no);
    let default_hash = hash_password(&default_password)?;
    let now = Utc::now();
    if let Some(existing) = User::find()
        .filter(users::Column::Username.eq(&student.student_no))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
    {
        let mut active: users::ActiveModel = existing.into();
        active.password_hash = Set(Some(default_hash));
        active.allow_password_login = Set(true);
        active.password_updated_at = Set(Some(now));
        active.must_change_password = Set(true);
        active.updated_at = Set(now);
        active
            .update(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    } else {
        let model = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            username: Set(student.student_no.clone()),
            display_name: Set(student.name.clone()),
            role: Set("student".to_string()),
            email: Set(None),
            password_hash: Set(Some(default_hash)),
            allow_password_login: Set(true),
            password_updated_at: Set(Some(now)),
            must_change_password: Set(true),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
        };
        users::Entity::insert(model)
            .exec_without_returning(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
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

    let usernames: Vec<String> = results.iter().map(|item| item.student_no.clone()).collect();
    let user_records = if usernames.is_empty() {
        Vec::new()
    } else {
        User::find()
            .filter(users::Column::Username.is_in(usernames))
            .all(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?
    };
    let mut allow_map = HashMap::new();
    for record in user_records {
        allow_map.insert(record.username, record.allow_password_login);
    }

    Ok(Json(
        results
            .into_iter()
            .map(|model| {
                let allow = allow_map.get(&model.student_no).copied().unwrap_or(false);
                crate::routes::students::StudentResponse::from_model(model, allow)
            })
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

/// 恢复已删除学生（仅管理员）。
pub async fn restore_student(
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
        return Ok(Json(serde_json::json!({ "restored": true })));
    }

    let mut active: students::ActiveModel = student.into();
    active.is_deleted = Set(false);
    active.updated_at = Set(Utc::now());
    active
        .update(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(serde_json::json!({ "restored": true })))
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

/// 恢复已删除竞赛记录（仅管理员）。
pub async fn restore_contest_record(
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
        return Ok(Json(serde_json::json!({ "restored": true })));
    }
    let mut active: contest_records::ActiveModel = record.into();
    active.is_deleted = Set(false);
    active.updated_at = Set(Utc::now());
    active
        .update(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(Json(serde_json::json!({ "restored": true })))
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

    let (file_bytes, fields) = read_upload_payload(&mut multipart).await?;
    let field_map = fields
        .get("field_map")
        .map(|value| serde_json::from_str::<HashMap<String, String>>(value))
        .transpose()
        .map_err(|_| AppError::bad_request("invalid field_map"))?;
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

    let header_index = build_header_index(range.rows().next());
    let base_index = build_contest_field_map(&header_index, field_map.as_ref())?;

    let custom_field_map = load_form_field_map(&state, "contest").await?;
    let reserved_headers = collect_reserved_headers_by_index(&header_index, &base_index);

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
            first_reviewer_id: Set(None),
            final_reviewer_id: Set(None),
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

        insert_custom_fields(
            &transaction,
            "contest",
            record_id,
            row,
            &header_index,
            &custom_field_map,
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

async fn read_upload_file(multipart: &mut Multipart) -> Result<(Vec<u8>, String), AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::bad_request("invalid multipart"))?
    {
        if field.name() == Some("file") {
            let name = field
                .file_name()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "template.xlsx".to_string());
            let bytes = field
                .bytes()
                .await
                .map_err(|_| AppError::bad_request("failed to read file"))?;
            return Ok((bytes.to_vec(), name));
        }
    }
    Err(AppError::bad_request("file field required"))
}

async fn read_upload_payload(
    multipart: &mut Multipart,
) -> Result<(Vec<u8>, HashMap<String, String>), AppError> {
    let mut file_bytes = None;
    let mut fields = HashMap::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::bad_request("invalid multipart"))?
    {
        let name = field.name().map(|value| value.to_string());
        match name.as_deref() {
            Some("file") => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|_| AppError::bad_request("failed to read file"))?;
                file_bytes = Some(bytes.to_vec());
            }
            Some(key) => {
                let value = field
                    .text()
                    .await
                    .map_err(|_| AppError::bad_request("failed to read field"))?;
                fields.insert(key.to_string(), value);
            }
            None => {}
        }
    }
    let file_bytes = file_bytes.ok_or_else(|| AppError::bad_request("file field required"))?;
    Ok((file_bytes, fields))
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

fn resolve_column_index(
    header_index: &HashMap<String, usize>,
    column: Option<&str>,
    fallback: &[&str],
) -> Option<usize> {
    if let Some(value) = column {
        let trimmed = value.trim();
        if let Some(idx) = parse_column_reference(trimmed) {
            return Some(idx);
        }
        if let Some(idx) = header_index.get(trimmed) {
            return Some(*idx);
        }
    }
    find_header_index(header_index, fallback)
}

fn build_contest_field_map(
    header_index: &HashMap<String, usize>,
    field_map: Option<&HashMap<String, String>>,
) -> Result<HashMap<String, usize>, AppError> {
    let mut result = HashMap::new();
    for (key, candidates) in CONTEST_IMPORT_HEADERS {
        let override_value = field_map.and_then(|map| map.get(key).map(|value| value.as_str()));
        let idx = resolve_column_index(header_index, override_value, candidates);
        let required = matches!(
            key,
            "student_no"
                | "contest_name"
                | "contest_level"
                | "contest_role"
                | "award_level"
                | "self_hours"
        );
        if required && idx.is_none() {
            return Err(AppError::bad_request("missing required header"));
        }
        if let Some(idx) = idx {
            result.insert(key.to_string(), idx);
        }
    }
    Ok(result)
}

fn collect_reserved_headers_by_index(
    header_index: &HashMap<String, usize>,
    base_index: &HashMap<String, usize>,
) -> Vec<String> {
    let mut reserved = Vec::new();
    for idx in base_index.values() {
        if let Some(name) = header_name_for_index(header_index, *idx) {
            reserved.push(name);
        }
    }
    reserved
}

fn header_name_for_index(
    header_index: &HashMap<String, usize>,
    index: usize,
) -> Option<String> {
    header_index
        .iter()
        .find_map(|(key, value)| (*value == index).then(|| key.clone()))
}

fn parse_column_reference(value: &str) -> Option<usize> {
    if value.is_empty() {
        return None;
    }
    if value.chars().all(|ch| ch.is_ascii_digit()) {
        let number = value.parse::<usize>().ok()?;
        return number.checked_sub(1);
    }
    if value.chars().all(|ch| ch.is_ascii_alphabetic()) {
        let mut index = 0usize;
        for ch in value.chars() {
            let upper = ch.to_ascii_uppercase();
            let offset = upper as u8;
            if offset < b'A' || offset > b'Z' {
                return None;
            }
            index = index * 26 + (offset - b'A' + 1) as usize;
        }
        return index.checked_sub(1);
    }
    None
}

fn normalize_category(value: &str, suffix: Option<&str>) -> String {
    let trimmed = value.trim();
    let normalized = match suffix {
        Some("class_contest") => trimmed
            .strip_suffix("类竞赛")
            .or_else(|| trimmed.strip_suffix("类"))
            .unwrap_or(trimmed),
        Some("class") => trimmed.strip_suffix("类").unwrap_or(trimmed),
        _ => trimmed,
    };
    normalized.trim().to_string()
}

fn export_template_to_response(template: ExportTemplateConfig) -> ExportTemplateResponse {
    ExportTemplateResponse {
        template_key: template.template_key,
        name: template.name,
        issues: template.issues,
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
