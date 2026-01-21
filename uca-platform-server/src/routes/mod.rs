//! HTTP 路由处理器。

use axum::{routing::{delete, get, post}, Router};

use crate::state::AppState;

pub mod auth;
pub mod attachments;
pub mod admin;
pub mod exports;
pub mod students;
pub mod records;
pub mod forms;

/// 构建应用路由。
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(auth::health))
        .route("/auth/bootstrap", post(auth::bootstrap_admin))
        .route("/auth/passkey/register/start", post(auth::passkey_register_start))
        .route("/auth/passkey/register/finish", post(auth::passkey_register_finish))
        .route("/auth/passkey/login/start", post(auth::passkey_login_start))
        .route("/auth/passkey/login/finish", post(auth::passkey_login_finish))
        .route("/auth/me", get(auth::current_user))
        .route("/auth/totp/enroll/start", post(auth::totp_enroll_start))
        .route("/auth/totp/enroll/finish", post(auth::totp_enroll_finish))
        .route("/auth/totp/verify", post(auth::totp_verify))
        .route("/auth/recovery/verify", post(auth::recovery_verify))
        .route("/auth/recovery/regenerate", post(auth::recovery_regenerate))
        .route("/auth/devices", get(auth::list_devices))
        .route("/auth/devices/:device_id", delete(auth::delete_device))
        .route("/forms/:form_type/fields", get(forms::list_form_fields_for_type))
        .route("/competitions", get(admin::list_competitions_public))
        .route("/students", post(students::create_student))
        .route("/students/query", post(students::list_students))
        .route("/students/import", post(students::import_students))
        .route("/records/volunteer", post(records::create_volunteer_record))
        .route("/records/contest", post(records::create_contest_record))
        .route("/records/volunteer/query", post(records::list_volunteer_records))
        .route("/records/contest/query", post(records::list_contest_records))
        .route("/records/volunteer/:record_id/review", post(records::review_volunteer_record))
        .route("/records/contest/:record_id/review", post(records::review_contest_record))
        .route("/attachments/volunteer/:record_id", post(attachments::upload_volunteer_attachment))
        .route("/attachments/contest/:record_id", post(attachments::upload_contest_attachment))
        .route("/signatures/:record_type/:record_id/:stage", post(attachments::upload_review_signature))
        .route("/export/summary/excel", post(exports::export_summary_excel))
        .route("/export/student/:student_no/excel", post(exports::export_student_excel))
        .route("/export/record/:record_type/:record_id/pdf", post(exports::export_record_pdf))
        .route("/admin/competitions", get(admin::list_competitions))
        .route("/admin/competitions", post(admin::create_competition))
        .route("/admin/competitions/import", post(admin::import_competitions))
        .route("/admin/form-fields", get(admin::list_form_fields))
        .route("/admin/form-fields", post(admin::create_form_field))
        .route("/admin/deleted/students", get(admin::list_deleted_students))
        .route("/admin/deleted/records/volunteer", get(admin::list_deleted_volunteer_records))
        .route("/admin/deleted/records/contest", get(admin::list_deleted_contest_records))
        .route("/admin/students/:student_no", delete(admin::delete_student))
        .route("/admin/records/volunteer/:record_id", delete(admin::delete_volunteer_record))
        .route("/admin/records/contest/:record_id", delete(admin::delete_contest_record))
        .route("/admin/purge/students/:student_no", delete(admin::purge_student))
        .route("/admin/purge/records/volunteer/:record_id", delete(admin::purge_volunteer_record))
        .route("/admin/purge/records/contest/:record_id", delete(admin::purge_contest_record))
        .route("/admin/records/volunteer/import", post(admin::import_volunteer_records))
        .route("/admin/records/contest/import", post(admin::import_contest_records))
        .with_state(state)
}
