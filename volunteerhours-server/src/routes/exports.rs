//! 导出 PDF / Excel 接口。

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use printpdf::{BuiltinFont, Image, ImageTransform, Mm, PdfDocument};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use std::io::{BufWriter, Cursor};
use std::path::Path as StdPath;
use uuid::Uuid;

use crate::{
    access::require_session_user,
    entities::{
        contest_records, review_signatures, students, volunteer_records, ContestRecord,
        ReviewSignature, Student, VolunteerRecord,
    },
    error::AppError,
    state::AppState,
};

/// 汇总导出筛选条件。
#[derive(Debug, Deserialize)]
pub struct ExportSummaryQuery {
    /// 院系筛选。
    pub department: Option<String>,
    /// 专业筛选。
    pub major: Option<String>,
    /// 班级筛选。
    pub class_name: Option<String>,
}

/// 导出学院/专业/班级汇总表。
pub async fn export_summary_excel(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(query): Json<ExportSummaryQuery>,
) -> Result<Response, AppError> {
    let user = require_session_user(&state, &jar).await?;
    if user.role != "admin" && user.role != "teacher" && user.role != "reviewer" {
        return Err(AppError::auth("forbidden"));
    }

    let mut finder = Student::find();
    if let Some(value) = query.department {
        finder = finder.filter(students::Column::Department.eq(value));
    }
    if let Some(value) = query.major {
        finder = finder.filter(students::Column::Major.eq(value));
    }
    if let Some(value) = query.class_name {
        finder = finder.filter(students::Column::ClassName.eq(value));
    }

    let students = finder
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut workbook = rust_xlsxwriter::Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .write_string(0, 0, "学号")
        .map_err(|_| AppError::internal("write excel failed"))?;
    worksheet
        .write_string(0, 1, "姓名")
        .map_err(|_| AppError::internal("write excel failed"))?;
    worksheet
        .write_string(0, 2, "班级")
        .map_err(|_| AppError::internal("write excel failed"))?;
    worksheet
        .write_string(0, 3, "个人自评学时")
        .map_err(|_| AppError::internal("write excel failed"))?;
    worksheet
        .write_string(0, 4, "审核通过学时")
        .map_err(|_| AppError::internal("write excel failed"))?;
    worksheet
        .write_string(0, 5, "备注")
        .map_err(|_| AppError::internal("write excel failed"))?;

    for (idx, student) in students.iter().enumerate() {
        let (self_hours, approved_hours, reason) =
            compute_student_hours(&state, student.id).await?;
        let row = (idx + 1) as u32;
        worksheet
            .write_string(row, 0, &student.student_no)
            .map_err(|_| AppError::internal("write excel failed"))?;
        worksheet
            .write_string(row, 1, &student.name)
            .map_err(|_| AppError::internal("write excel failed"))?;
        worksheet
            .write_string(row, 2, &student.class_name)
            .map_err(|_| AppError::internal("write excel failed"))?;
        worksheet
            .write_number(row, 3, self_hours as f64)
            .map_err(|_| AppError::internal("write excel failed"))?;
        worksheet
            .write_number(row, 4, approved_hours as f64)
            .map_err(|_| AppError::internal("write excel failed"))?;
        worksheet
            .write_string(row, 5, &reason)
            .map_err(|_| AppError::internal("write excel failed"))?;
    }

    let buffer = workbook
        .save_to_buffer()
        .map_err(|_| AppError::internal("save excel failed"))?;

    Ok(file_response(
        "summary.xlsx",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        buffer,
    ))
}

/// 导出个人学时专项表（管理员/教师/本人）。
pub async fn export_student_excel(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(student_no): Path<String>,
) -> Result<Response, AppError> {
    let user = require_session_user(&state, &jar).await?;
    if user.role == "student" && user.username != student_no {
        return Err(AppError::auth("forbidden"));
    }

    let student = Student::find()
        .filter(students::Column::StudentNo.eq(&student_no))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("student not found"))?;

    let (self_hours, approved_hours, reason) =
        compute_student_hours(&state, student.id).await?;

    let mut workbook = rust_xlsxwriter::Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .write_string(0, 0, "学号")
        .map_err(|_| AppError::internal("write excel failed"))?;
    worksheet
        .write_string(0, 1, "姓名")
        .map_err(|_| AppError::internal("write excel failed"))?;
    worksheet
        .write_string(0, 2, "个人自评学时")
        .map_err(|_| AppError::internal("write excel failed"))?;
    worksheet
        .write_string(0, 3, "审核通过学时")
        .map_err(|_| AppError::internal("write excel failed"))?;
    worksheet
        .write_string(0, 4, "备注")
        .map_err(|_| AppError::internal("write excel failed"))?;

    worksheet
        .write_string(1, 0, &student.student_no)
        .map_err(|_| AppError::internal("write excel failed"))?;
    worksheet
        .write_string(1, 1, &student.name)
        .map_err(|_| AppError::internal("write excel failed"))?;
    worksheet
        .write_number(1, 2, self_hours as f64)
        .map_err(|_| AppError::internal("write excel failed"))?;
    worksheet
        .write_number(1, 3, approved_hours as f64)
        .map_err(|_| AppError::internal("write excel failed"))?;
    worksheet
        .write_string(1, 4, &reason)
        .map_err(|_| AppError::internal("write excel failed"))?;

    let buffer = workbook
        .save_to_buffer()
        .map_err(|_| AppError::internal("save excel failed"))?;

    Ok(file_response(
        format!("{}-summary.xlsx", student.student_no),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        buffer,
    ))
}

/// 导出记录 PDF（志愿/竞赛）。
pub async fn export_record_pdf(
    State(state): State<AppState>,
    jar: CookieJar,
    Path((record_type, record_id)): Path<(String, Uuid)>,
) -> Result<Response, AppError> {
    let user = require_session_user(&state, &jar).await?;

    let (student, summary) = match record_type.as_str() {
        "volunteer" => {
            let record = VolunteerRecord::find_by_id(record_id)
                .one(&state.db)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?
                .ok_or_else(|| AppError::not_found("record not found"))?;
            let student = Student::find_by_id(record.student_id)
                .one(&state.db)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?
                .ok_or_else(|| AppError::not_found("student not found"))?;

            if user.role == "student" && user.username != student.student_no {
                return Err(AppError::auth("forbidden"));
            }
            let summary = vec![
                ("记录类型", "志愿服务".to_string()),
                ("标题", record.title),
                ("描述", record.description),
                ("自评学时", record.self_hours.to_string()),
                (
                    "初审学时",
                    record.first_review_hours.map_or("".to_string(), |v| v.to_string()),
                ),
                (
                    "复审学时",
                    record.final_review_hours.map_or("".to_string(), |v| v.to_string()),
                ),
                ("状态", record.status),
                (
                    "不通过原因",
                    record.rejection_reason.unwrap_or_default(),
                ),
            ];
            (student, summary)
        }
        "contest" => {
            let record = ContestRecord::find_by_id(record_id)
                .one(&state.db)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?
                .ok_or_else(|| AppError::not_found("record not found"))?;
            let student = Student::find_by_id(record.student_id)
                .one(&state.db)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?
                .ok_or_else(|| AppError::not_found("student not found"))?;

            if user.role == "student" && user.username != student.student_no {
                return Err(AppError::auth("forbidden"));
            }
            let summary = vec![
                ("记录类型", "竞赛获奖".to_string()),
                ("竞赛名称", record.contest_name),
                ("获奖等级", record.award_level),
                ("自评学时", record.self_hours.to_string()),
                (
                    "初审学时",
                    record.first_review_hours.map_or("".to_string(), |v| v.to_string()),
                ),
                (
                    "复审学时",
                    record.final_review_hours.map_or("".to_string(), |v| v.to_string()),
                ),
                ("状态", record.status),
                (
                    "不通过原因",
                    record.rejection_reason.unwrap_or_default(),
                ),
            ];
            (student, summary)
        }
        _ => return Err(AppError::bad_request("invalid record type")),
    };

    let signatures = ReviewSignature::find()
        .filter(review_signatures::Column::RecordType.eq(record_type.clone()))
        .filter(review_signatures::Column::RecordId.eq(record_id))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let (doc, page1, layer1) = PdfDocument::new("record", Mm(210.0), Mm(297.0), "Layer 1");
    let layer = doc.get_page(page1).get_layer(layer1);
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|_| AppError::internal("load font failed"))?;

    layer.use_text(
        format!("学生: {} ({})", student.name, student.student_no),
        14.0,
        Mm(20.0),
        Mm(280.0),
        &font,
    );

    let mut y = 260.0;
    for (label, value) in summary {
        layer.use_text(
            format!("{label}: {value}"),
            12.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 12.0;
    }

    let mut y_sig = 140.0;
    for sig in signatures {
        layer.use_text(
            format!("签名({}): {}", sig.stage, sig.signature_path),
            10.0,
            Mm(20.0),
            Mm(y_sig),
            &font,
        );
        y_sig -= 10.0;

        if let Some(image) = load_signature_image(&sig.signature_path) {
            let transform = ImageTransform {
                translate_x: Some(Mm(20.0)),
                translate_y: Some(Mm(y_sig)),
                scale_x: Some(0.3),
                scale_y: Some(0.3),
                ..Default::default()
            };
            image.add_to_layer(layer.clone(), transform);
            y_sig -= 30.0;
        }
    }

    let mut writer = BufWriter::new(Cursor::new(Vec::new()));
    doc.save(&mut writer)
        .map_err(|_| AppError::internal("save pdf failed"))?;
    let cursor = writer
        .into_inner()
        .map_err(|_| AppError::internal("save pdf failed"))?;
    let buffer = cursor.into_inner();

    Ok(file_response(
        format!("record-{}.pdf", record_id),
        "application/pdf",
        buffer,
    ))
}

async fn compute_student_hours(
    state: &AppState,
    student_id: Uuid,
) -> Result<(i32, i32, String), AppError> {
    let volunteer = VolunteerRecord::find()
        .filter(volunteer_records::Column::StudentId.eq(student_id))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    let contest = ContestRecord::find()
        .filter(contest_records::Column::StudentId.eq(student_id))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut self_hours = 0;
    let mut approved = 0;
    let mut reasons = Vec::new();

    for record in volunteer {
        self_hours += record.self_hours;
        if record.status == "final_reviewed" {
            approved += record.final_review_hours.unwrap_or(0);
        }
        if record.status == "rejected" {
            if let Some(reason) = record.rejection_reason {
                reasons.push(reason);
            }
        }
    }

    for record in contest {
        self_hours += record.self_hours;
        if record.status == "final_reviewed" {
            approved += record.final_review_hours.unwrap_or(0);
        }
        if record.status == "rejected" {
            if let Some(reason) = record.rejection_reason {
                reasons.push(reason);
            }
        }
    }

    Ok((self_hours, approved, reasons.join(";")))
}

fn file_response(name: impl Into<String>, mime: &str, bytes: Vec<u8>) -> Response {
    let mut response = bytes.into_response();
    let name = name.into();
    let headers = response.headers_mut();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        mime.parse().unwrap_or_else(|_| "application/octet-stream".parse().unwrap()),
    );
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{name}\"")
            .parse()
            .unwrap(),
    );
    response
}

fn load_signature_image(path: &str) -> Option<Image> {
    let path = StdPath::new(path);
    if !path.exists() {
        return None;
    }
    let image = image::io::Reader::open(path).ok()?.decode().ok()?;
    Some(Image::from_dynamic_image(&image))
}
