//! 导出 PDF / Excel 接口。

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use printpdf::{BuiltinFont, Color, Image, ImageTransform, Line, Mm, PdfDocument, Point, Rgb};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{BufWriter, Cursor};
use std::path::Path as StdPath;
use std::process::Command;
use uuid::Uuid;

use crate::{
    access::require_session_user,
    entities::{
        contest_records, form_field_values, form_fields, review_signatures, students,
        ContestRecord, FormField, FormFieldValue, ReviewSignature, Student, UserSignature,
    },
    error::AppError,
    export_template::render_template_to_xlsx,
    labor_hours::{compute_recommended_hours, load_labor_hour_rules},
    state::AppState,
    templates::export_template_file_path,
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
        .filter(students::Column::IsDeleted.eq(false))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let fields = load_export_fields(&state, "summary").await?;
    let export_fields = if fields.is_empty() {
        default_summary_fields()
    } else {
        fields
    };

    let mut workbook = rust_xlsxwriter::Workbook::new();
    let worksheet = workbook.add_worksheet();
    for (idx, field) in export_fields.iter().enumerate() {
        worksheet
            .write_string(0, idx as u16, &field.label)
            .map_err(|_| AppError::internal("write excel failed"))?;
    }

    for (idx, student) in students.iter().enumerate() {
        let (self_hours, approved_hours, reason) =
            compute_student_hours(&state, student.id).await?;
        let row = (idx + 1) as u32;
        for (col, field) in export_fields.iter().enumerate() {
            let value = resolve_export_value(field.field_key.as_str(), student, self_hours, approved_hours, &reason);
            write_cell(worksheet, row, col as u16, &value)?;
        }
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
        .filter(students::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("student not found"))?;

    let (self_hours, approved_hours, reason) =
        compute_student_hours(&state, student.id).await?;

    let fields = load_export_fields(&state, "student_export").await?;
    let export_fields = if fields.is_empty() {
        default_student_fields()
    } else {
        fields
    };

    let mut workbook = rust_xlsxwriter::Workbook::new();
    let worksheet = workbook.add_worksheet();
    for (idx, field) in export_fields.iter().enumerate() {
        worksheet
            .write_string(0, idx as u16, &field.label)
            .map_err(|_| AppError::internal("write excel failed"))?;
    }

    for (col, field) in export_fields.iter().enumerate() {
        let value = resolve_export_value(field.field_key.as_str(), &student, self_hours, approved_hours, &reason);
        write_cell(worksheet, 1, col as u16, &value)?;
    }

    let buffer = workbook
        .save_to_buffer()
        .map_err(|_| AppError::internal("save excel failed"))?;

    Ok(file_response(
        format!("{}-summary.xlsx", student.student_no),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        buffer,
    ))
}

/// 导出劳动教育学时汇总表（Excel）。
pub async fn export_labor_hours_summary_excel(
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
        .filter(students::Column::IsDeleted.eq(false))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let fields = load_export_fields(&state, "labor_hours_excel").await?;
    let export_fields = if fields.is_empty() {
        default_labor_hours_excel_fields()
    } else {
        fields
    };

    let mut workbook = rust_xlsxwriter::Workbook::new();
    let worksheet = workbook.add_worksheet();
    for (idx, field) in export_fields.iter().enumerate() {
        worksheet
            .write_string(0, idx as u16, &field.label)
            .map_err(|_| AppError::internal("write excel failed"))?;
    }

    for (idx, student) in students.iter().enumerate() {
        let (self_hours, approved_hours, reason) =
            compute_student_hours(&state, student.id).await?;
        let row = (idx + 1) as u32;
        for (col, field) in export_fields.iter().enumerate() {
            let value = resolve_labor_hours_export_value(
                field.field_key.as_str(),
                idx + 1,
                student,
                self_hours,
                approved_hours,
                &reason,
            );
            write_cell(worksheet, row, col as u16, &value)?;
        }
    }

    let buffer = workbook
        .save_to_buffer()
        .map_err(|_| AppError::internal("save excel failed"))?;

    Ok(file_response(
        "labor-hours-summary.xlsx",
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
        "contest" => {
            let record = ContestRecord::find()
                .filter(contest_records::Column::Id.eq(record_id))
                .filter(contest_records::Column::IsDeleted.eq(false))
                .one(&state.db)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?
                .ok_or_else(|| AppError::not_found("record not found"))?;
            let student = Student::find_by_id(record.student_id)
                .filter(students::Column::IsDeleted.eq(false))
                .one(&state.db)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?
                .ok_or_else(|| AppError::not_found("student not found"))?;

            if user.role == "student" && user.username != student.student_no {
                return Err(AppError::auth("forbidden"));
            }
            let rule = load_labor_hour_rules(&state).await?;
            let recommended = compute_recommended_hours(
                rule,
                record.contest_category.as_deref(),
                record.contest_level.as_deref(),
                record.contest_role.as_deref(),
            );
            let summary = vec![
                ("记录类型".to_string(), "竞赛获奖".to_string()),
                (
                    "竞赛年份".to_string(),
                    record
                        .contest_year
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                ),
                (
                    "竞赛类型".to_string(),
                    record.contest_category.clone().unwrap_or_default(),
                ),
                ("竞赛名称".to_string(), record.contest_name),
                (
                    "竞赛级别".to_string(),
                    record.contest_level.clone().unwrap_or_default(),
                ),
                (
                    "竞赛角色".to_string(),
                    record.contest_role.clone().unwrap_or_default(),
                ),
                ("获奖等级".to_string(), record.award_level),
                (
                    "获奖时间".to_string(),
                    record
                        .award_date
                        .map(|value| value.to_rfc3339())
                        .unwrap_or_default(),
                ),
                ("自评学时".to_string(), record.self_hours.to_string()),
                ("推荐学时".to_string(), recommended.to_string()),
                (
                    "初审学时".to_string(),
                    record.first_review_hours.map_or("".to_string(), |v| v.to_string()),
                ),
                (
                    "复审学时".to_string(),
                    record.final_review_hours.map_or("".to_string(), |v| v.to_string()),
                ),
                ("状态".to_string(), record.status),
                (
                    "不通过原因".to_string(),
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

    let mut summary = summary;
    let custom_fields = load_custom_fields(&state, &record_type, record_id).await?;
    for field in custom_fields {
        summary.push((field.label, field.value));
    }

    let (doc, page1, layer1) = PdfDocument::new("record", Mm(210.0), Mm(297.0), "Layer 1");
    let mut layer = doc.get_page(page1).get_layer(layer1);
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|_| AppError::internal("load font failed"))?;

    let mut current_page = 1;
    let mut y: f32 = 280.0;
    layer.set_outline_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));

    layer.use_text(
        "审核记录详情",
        16.0,
        Mm(20.0),
        Mm(y),
        &font,
    );
    y -= 12.0;
    layer.use_text(
        format!("学生: {} ({})", student.name, student.student_no),
        12.0,
        Mm(20.0),
        Mm(y),
        &font,
    );
    y -= 10.0;
    y = draw_table_header(&layer, &font, y);

    for (label, value) in summary {
        let lines = wrap_text(&value, 26);
        let row_height = 8.0 * lines.len() as f32 + 4.0;
        if y - row_height < 30.0 {
            let (page, layer_id) = doc.add_page(Mm(210.0), Mm(297.0), "Layer");
            layer = doc.get_page(page).get_layer(layer_id);
            layer.set_outline_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
            current_page += 1;
            y = 280.0;
            layer.use_text(
                format!("审核记录详情（续页 {current_page}）"),
                14.0,
                Mm(20.0),
                Mm(y),
                &font,
            );
            y -= 10.0;
            y = draw_table_header(&layer, &font, y);
        }
        y = draw_table_row(&layer, &font, y, &label, &lines);
    }

    y -= 8.0;
    if y < 60.0 {
        let (page, layer_id) = doc.add_page(Mm(210.0), Mm(297.0), "Layer");
        layer = doc.get_page(page).get_layer(layer_id);
        layer.set_outline_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
        current_page += 1;
        y = 280.0;
        layer.use_text(
            format!("审核记录详情（续页 {current_page}）"),
            14.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 14.0;
    }

    layer.use_text("审核签名", 12.0, Mm(20.0), Mm(y), &font);
    y -= 8.0;

    for sig in signatures {
        let label = format!("{}签名", if sig.stage == "first" { "初审" } else { "复审" });
        if y < 50.0 {
            let (page, layer_id) = doc.add_page(Mm(210.0), Mm(297.0), "Layer");
            layer = doc.get_page(page).get_layer(layer_id);
            layer.set_outline_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
            current_page += 1;
            y = 280.0;
            layer.use_text(
                format!("审核记录详情（续页 {current_page}）"),
                14.0,
                Mm(20.0),
                Mm(y),
                &font,
            );
            y -= 14.0;
            layer.use_text("审核签名", 12.0, Mm(20.0), Mm(y), &font);
            y -= 8.0;
        }
        layer.use_text(label, 10.0, Mm(20.0), Mm(y), &font);
        if let Some(image) = load_signature_image(&sig.signature_path) {
            let transform = ImageTransform {
                translate_x: Some(Mm(60.0)),
                translate_y: Some(Mm(y - 6.0)),
                scale_x: Some(0.25),
                scale_y: Some(0.25),
                ..Default::default()
            };
            image.add_to_layer(layer.clone(), transform);
        } else {
            layer.use_text("未找到签名文件", 10.0, Mm(60.0), Mm(y), &font);
        }
        y -= 24.0;
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

/// 导出劳动教育学时认定表 PDF（每学生一份）。
pub async fn export_labor_hours_pdf(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(student_no): Path<String>,
) -> Result<Response, AppError> {
    let user = require_session_user(&state, &jar).await?;
    if user.role == "student" && user.username != student_no {
        return Err(AppError::auth("forbidden"));
    }
    if user.role != "student" && user.role != "admin" && user.role != "teacher" && user.role != "reviewer" {
        return Err(AppError::auth("forbidden"));
    }

    let student = Student::find()
        .filter(students::Column::StudentNo.eq(&student_no))
        .filter(students::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::not_found("student not found"))?;

    let records = ContestRecord::find()
        .filter(contest_records::Column::StudentId.eq(student.id))
        .filter(contest_records::Column::IsDeleted.eq(false))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let form_fields = load_export_fields(&state, "contest").await?;
    let record_ids: Vec<Uuid> = records.iter().map(|record| record.id).collect();
    let custom_fields = load_custom_field_values(&state, "contest", &record_ids, &form_fields).await?;
    let (self_hours, approved_hours, reason) =
        compute_student_hours(&state, student.id).await?;

    let rule_config = load_labor_hour_rules(&state).await?;
    let signature_bundle = load_reviewer_signatures(&state, &records).await?;

    let template_path = export_template_file_path(&state, "labor_hours");
    if !template_path.exists() {
        return Err(AppError::bad_request("export template not configured"));
    }

    let single_values = build_single_values(
        &student,
        self_hours,
        approved_hours,
        &reason,
        &signature_bundle,
    );
    let list_values = build_list_values(&records, &custom_fields, rule_config);

    let temp_dir = tempfile::tempdir()
        .map_err(|_| AppError::internal("create temp dir failed"))?;
    let output_xlsx = temp_dir.path().join("labor_hours.xlsx");
    render_template_to_xlsx(&template_path, &output_xlsx, &single_values, &list_values)?;
    let buffer = convert_xlsx_to_pdf(&state.config.libreoffice_path, &output_xlsx, temp_dir.path())?;

    Ok(file_response(
        format!("{}-labor-hours.pdf", student.student_no),
        "application/pdf",
        buffer,
    ))
}

async fn compute_student_hours(
    state: &AppState,
    student_id: Uuid,
) -> Result<(i32, i32, String), AppError> {
    let contest = ContestRecord::find()
        .filter(contest_records::Column::StudentId.eq(student_id))
        .filter(contest_records::Column::IsDeleted.eq(false))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut self_hours = 0;
    let mut approved = 0;
    let mut reasons = Vec::new();

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

struct SignatureBundle {
    first: Option<String>,
    final_review: Option<String>,
}

async fn load_reviewer_signatures(
    state: &AppState,
    records: &[contest_records::Model],
) -> Result<SignatureBundle, AppError> {
    let mut first: Option<(chrono::DateTime<chrono::Utc>, Uuid)> = None;
    let mut final_review: Option<(chrono::DateTime<chrono::Utc>, Uuid)> = None;

    for record in records {
        if let Some(user_id) = record.first_reviewer_id {
            let replace = first
                .as_ref()
                .map(|(time, _)| record.updated_at > *time)
                .unwrap_or(true);
            if replace {
                first = Some((record.updated_at, user_id));
            }
        }
        if let Some(user_id) = record.final_reviewer_id {
            let replace = final_review
                .as_ref()
                .map(|(time, _)| record.updated_at > *time)
                .unwrap_or(true);
            if replace {
                final_review = Some((record.updated_at, user_id));
            }
        }
    }

    let first_path = if let Some((_, user_id)) = first {
        UserSignature::find_by_id(user_id)
            .one(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?
            .map(|model| model.signature_path)
    } else {
        None
    };
    let final_path = if let Some((_, user_id)) = final_review {
        UserSignature::find_by_id(user_id)
            .one(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?
            .map(|model| model.signature_path)
    } else {
        None
    };

    Ok(SignatureBundle {
        first: first_path,
        final_review: final_path,
    })
}

fn build_single_values(
    student: &students::Model,
    self_hours: i32,
    approved_hours: i32,
    reason: &str,
    signatures: &SignatureBundle,
) -> HashMap<String, String> {
    let mut values = HashMap::new();
    values.insert("student_no".to_string(), student.student_no.clone());
    values.insert("name".to_string(), student.name.clone());
    values.insert("gender".to_string(), student.gender.clone());
    values.insert("department".to_string(), student.department.clone());
    values.insert("major".to_string(), student.major.clone());
    values.insert("class_name".to_string(), student.class_name.clone());
    values.insert("phone".to_string(), student.phone.clone());
    values.insert("total_self_hours".to_string(), self_hours.to_string());
    values.insert("total_approved_hours".to_string(), approved_hours.to_string());
    values.insert("total_reason".to_string(), reason.to_string());
    if let Some(path) = signatures.first.as_ref() {
        values.insert("first_signature_path".to_string(), path.clone());
        values.insert("first_signature_image".to_string(), path.clone());
    }
    if let Some(path) = signatures.final_review.as_ref() {
        values.insert("final_signature_path".to_string(), path.clone());
        values.insert("final_signature_image".to_string(), path.clone());
    }
    values
}

fn build_list_values(
    records: &[contest_records::Model],
    custom_fields: &HashMap<Uuid, HashMap<String, String>>,
    rule_config: crate::labor_hours::LaborHourRuleConfig,
) -> Vec<HashMap<String, String>> {
    let mut items = Vec::new();
    for record in records {
        let recommended = compute_recommended_hours(
            rule_config,
            record.contest_category.as_deref(),
            record.contest_level.as_deref(),
            record.contest_role.as_deref(),
        );
        let mut map = HashMap::new();
        map.insert(
            "contest_year".to_string(),
            record
                .contest_year
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        map.insert(
            "contest_category".to_string(),
            record.contest_category.clone().unwrap_or_default(),
        );
        map.insert("contest_name".to_string(), record.contest_name.clone());
        map.insert(
            "contest_level".to_string(),
            record.contest_level.clone().unwrap_or_default(),
        );
        map.insert(
            "contest_role".to_string(),
            record.contest_role.clone().unwrap_or_default(),
        );
        map.insert("award_level".to_string(), record.award_level.clone());
        map.insert(
            "award_date".to_string(),
            record
                .award_date
                .map(|value| value.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
        );
        map.insert("self_hours".to_string(), record.self_hours.to_string());
        map.insert(
            "first_review_hours".to_string(),
            record
                .first_review_hours
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        map.insert(
            "final_review_hours".to_string(),
            record
                .final_review_hours
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        map.insert(
            "approved_hours".to_string(),
            record
                .final_review_hours
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        map.insert("status".to_string(), record.status.clone());
        map.insert(
            "rejection_reason".to_string(),
            record.rejection_reason.clone().unwrap_or_default(),
        );
        map.insert("recommended_hours".to_string(), recommended.to_string());

        if let Some(custom) = custom_fields.get(&record.id) {
            for (key, value) in custom {
                map.insert(format!("custom.{key}"), value.clone());
            }
        }
        items.push(map);
    }
    items
}

fn convert_xlsx_to_pdf(
    libreoffice_path: &str,
    input_path: &StdPath,
    output_dir: &StdPath,
) -> Result<Vec<u8>, AppError> {
    // 测试环境使用内置简易 PDF，避免依赖外部转换程序。
    if libreoffice_path == "internal" {
        let (doc, page, layer) =
            PdfDocument::new("劳动教育学时认定表", Mm(210.0), Mm(297.0), "Layer");
        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|_| AppError::internal("load font failed"))?;
        let layer = doc.get_page(page).get_layer(layer);
        layer.use_text("模板转换未启用（internal）", 12.0, Mm(20.0), Mm(280.0), &font);
        let mut writer = BufWriter::new(Cursor::new(Vec::new()));
        doc.save(&mut writer)
            .map_err(|_| AppError::internal("save pdf failed"))?;
        let cursor = writer
            .into_inner()
            .map_err(|_| AppError::internal("save pdf failed"))?;
        return Ok(cursor.into_inner());
    }

    let status = Command::new(libreoffice_path)
        .arg("--headless")
        .arg("--convert-to")
        .arg("pdf")
        .arg("--outdir")
        .arg(output_dir)
        .arg(input_path)
        .status()
        .map_err(|_| AppError::internal("run libreoffice failed"))?;
    if !status.success() {
        return Err(AppError::internal("libreoffice conversion failed"));
    }
    let file_stem = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| AppError::internal("invalid template filename"))?;
    let pdf_path = output_dir.join(format!("{file_stem}.pdf"));
    std::fs::read(&pdf_path).map_err(|_| AppError::internal("read pdf failed"))
}

async fn load_custom_field_values(
    state: &AppState,
    record_type: &str,
    record_ids: &[Uuid],
    fields: &[ExportField],
) -> Result<HashMap<Uuid, HashMap<String, String>>, AppError> {
    if record_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let mut field_keys = HashMap::new();
    for field in fields {
        field_keys.insert(field.field_key.as_str(), true);
    }

    let values = FormFieldValue::find()
        .filter(form_field_values::Column::RecordType.eq(record_type))
        .filter(form_field_values::Column::RecordId.is_in(record_ids.iter().cloned()))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut grouped: HashMap<Uuid, HashMap<String, String>> = HashMap::new();
    for value in values {
        if !field_keys.contains_key(value.field_key.as_str()) {
            continue;
        }
        grouped
            .entry(value.record_id)
            .or_default()
            .insert(value.field_key, value.value);
    }
    Ok(grouped)
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

#[derive(Clone)]
struct ExportField {
    field_key: String,
    label: String,
    order_index: i32,
}

async fn load_export_fields(state: &AppState, form_type: &str) -> Result<Vec<ExportField>, AppError> {
    let mut fields = FormField::find()
        .filter(form_fields::Column::FormType.eq(form_type))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
        .into_iter()
        .map(|field| ExportField {
            field_key: field.field_key,
            label: field.label,
            order_index: field.order_index,
        })
        .collect::<Vec<_>>();
    fields.sort_by_key(|item| item.order_index);
    Ok(fields)
}

fn default_summary_fields() -> Vec<ExportField> {
    vec![
        ExportField { field_key: "student_no".to_string(), label: "学号".to_string(), order_index: 1 },
        ExportField { field_key: "name".to_string(), label: "姓名".to_string(), order_index: 2 },
        ExportField { field_key: "class_name".to_string(), label: "班级".to_string(), order_index: 3 },
        ExportField { field_key: "self_hours".to_string(), label: "个人自评学时".to_string(), order_index: 4 },
        ExportField { field_key: "approved_hours".to_string(), label: "审核通过学时".to_string(), order_index: 5 },
        ExportField { field_key: "reason".to_string(), label: "备注".to_string(), order_index: 6 },
    ]
}

fn default_student_fields() -> Vec<ExportField> {
    vec![
        ExportField { field_key: "student_no".to_string(), label: "学号".to_string(), order_index: 1 },
        ExportField { field_key: "name".to_string(), label: "姓名".to_string(), order_index: 2 },
        ExportField { field_key: "self_hours".to_string(), label: "个人自评学时".to_string(), order_index: 3 },
        ExportField { field_key: "approved_hours".to_string(), label: "审核通过学时".to_string(), order_index: 4 },
        ExportField { field_key: "reason".to_string(), label: "备注".to_string(), order_index: 5 },
    ]
}

fn default_labor_hours_excel_fields() -> Vec<ExportField> {
    vec![
        ExportField { field_key: "index".to_string(), label: "序号".to_string(), order_index: 1 },
        ExportField { field_key: "major".to_string(), label: "专业".to_string(), order_index: 2 },
        ExportField { field_key: "class_name".to_string(), label: "班级".to_string(), order_index: 3 },
        ExportField { field_key: "student_no".to_string(), label: "学号".to_string(), order_index: 4 },
        ExportField { field_key: "name".to_string(), label: "姓名".to_string(), order_index: 5 },
        ExportField { field_key: "planned_hours".to_string(), label: "拟加学时".to_string(), order_index: 6 },
        ExportField {
            field_key: "module_hours".to_string(),
            label: "生产劳动教育模块学时（不少于4学时）".to_string(),
            order_index: 7,
        },
        ExportField { field_key: "reason".to_string(), label: "备注".to_string(), order_index: 8 },
    ]
}

fn resolve_export_value(
    field_key: &str,
    student: &students::Model,
    self_hours: i32,
    approved_hours: i32,
    reason: &str,
) -> ExportValue {
    match field_key {
        "student_no" => ExportValue::Text(student.student_no.clone()),
        "name" => ExportValue::Text(student.name.clone()),
        "gender" => ExportValue::Text(student.gender.clone()),
        "department" => ExportValue::Text(student.department.clone()),
        "major" => ExportValue::Text(student.major.clone()),
        "class_name" => ExportValue::Text(student.class_name.clone()),
        "phone" => ExportValue::Text(student.phone.clone()),
        "self_hours" => ExportValue::Number(self_hours as f64),
        "approved_hours" => ExportValue::Number(approved_hours as f64),
        "reason" => ExportValue::Text(reason.to_string()),
        _ => ExportValue::Text(String::new()),
    }
}

fn resolve_labor_hours_export_value(
    field_key: &str,
    index: usize,
    student: &students::Model,
    self_hours: i32,
    approved_hours: i32,
    reason: &str,
) -> ExportValue {
    match field_key {
        "index" => ExportValue::Number(index as f64),
        "student_no" => ExportValue::Text(student.student_no.clone()),
        "name" => ExportValue::Text(student.name.clone()),
        "gender" => ExportValue::Text(student.gender.clone()),
        "department" => ExportValue::Text(student.department.clone()),
        "major" => ExportValue::Text(student.major.clone()),
        "class_name" => ExportValue::Text(student.class_name.clone()),
        "phone" => ExportValue::Text(student.phone.clone()),
        "planned_hours" => ExportValue::Number(self_hours as f64),
        "module_hours" => ExportValue::Number(approved_hours as f64),
        "reason" => ExportValue::Text(reason.to_string()),
        _ => ExportValue::Text(String::new()),
    }
}

enum ExportValue {
    Text(String),
    Number(f64),
}

fn write_cell(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    value: &ExportValue,
) -> Result<(), AppError> {
    match value {
        ExportValue::Text(text) => worksheet
            .write_string(row, col, text)
            .map(|_| ())
            .map_err(|_| AppError::internal("write excel failed")),
        ExportValue::Number(number) => worksheet
            .write_number(row, col, *number)
            .map(|_| ())
            .map_err(|_| AppError::internal("write excel failed")),
    }
}

struct CustomFieldEntry {
    label: String,
    value: String,
    order_index: i32,
}

async fn load_custom_fields(
    state: &AppState,
    record_type: &str,
    record_id: Uuid,
) -> Result<Vec<CustomFieldEntry>, AppError> {
    let fields = FormField::find()
        .filter(form_fields::Column::FormType.eq(record_type))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    if fields.is_empty() {
        return Ok(Vec::new());
    }
    let mut field_map = std::collections::HashMap::new();
    for field in fields {
        field_map.insert(
            field.field_key.clone(),
            (field.label.clone(), field.order_index),
        );
    }

    let values = FormFieldValue::find()
        .filter(form_field_values::Column::RecordType.eq(record_type))
        .filter(form_field_values::Column::RecordId.eq(record_id))
        .all(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let mut result = Vec::new();
    for value in values {
        let (label, order_index) = field_map
            .get(&value.field_key)
            .cloned()
            .unwrap_or_else(|| (value.field_key.clone(), 0));
        result.push(CustomFieldEntry {
            label,
            value: value.value,
            order_index,
        });
    }
    result.sort_by_key(|item| item.order_index);
    Ok(result)
}

fn draw_table_header(
    layer: &printpdf::PdfLayerReference,
    font: &printpdf::IndirectFontRef,
    y: f32,
) -> f32 {
    let left: f32 = 20.0;
    let right: f32 = 190.0;
    let mid: f32 = 70.0;
    let header_height: f32 = 10.0;
    draw_line(layer, left, y, right, y);
    draw_line(layer, left, y - header_height, right, y - header_height);
    draw_line(layer, left, y, left, y - header_height);
    draw_line(layer, mid, y, mid, y - header_height);
    draw_line(layer, right, y, right, y - header_height);
    layer.use_text("字段", 10.0, Mm(left + 2.0), Mm(y - 7.0), font);
    layer.use_text("内容", 10.0, Mm(mid + 2.0), Mm(y - 7.0), font);
    y - header_height
}

fn draw_table_row(
    layer: &printpdf::PdfLayerReference,
    font: &printpdf::IndirectFontRef,
    y: f32,
    label: &str,
    lines: &[String],
) -> f32 {
    let left: f32 = 20.0;
    let right: f32 = 190.0;
    let mid: f32 = 70.0;
    let row_height: f32 = 8.0 * lines.len() as f32 + 4.0;
    let top = y;
    let bottom = y - row_height;
    draw_line(layer, left, top, right, top);
    draw_line(layer, left, bottom, right, bottom);
    draw_line(layer, left, top, left, bottom);
    draw_line(layer, mid, top, mid, bottom);
    draw_line(layer, right, top, right, bottom);
    layer.use_text(label, 10.0, Mm(left + 2.0), Mm(top - 6.0), font);
    for (idx, line) in lines.iter().enumerate() {
        let offset = 6.0 + idx as f32 * 8.0;
        layer.use_text(line, 10.0, Mm(mid + 2.0), Mm(top - offset), font);
    }
    bottom
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if current.chars().count() >= max_chars {
            lines.push(current);
            current = String::new();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn draw_line(layer: &printpdf::PdfLayerReference, x1: f32, y1: f32, x2: f32, y2: f32) {
    let line = Line {
        points: vec![(Point::new(Mm(x1), Mm(y1)), false), (Point::new(Mm(x2), Mm(y2)), false)],
        is_closed: false,
    };
    layer.add_line(line);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn build_student() -> students::Model {
        students::Model {
            id: Uuid::new_v4(),
            student_no: "2023001".to_string(),
            name: "张三".to_string(),
            gender: "男".to_string(),
            department: "信息学院".to_string(),
            major: "软件工程".to_string(),
            class_name: "软工1班".to_string(),
            phone: "13800000000".to_string(),
            is_deleted: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn wrap_text_handles_empty() {
        let lines = wrap_text("", 4);
        assert_eq!(lines, vec!["".to_string()]);
    }

    #[test]
    fn wrap_text_splits_by_max_chars() {
        let lines = wrap_text("abcdef", 2);
        assert_eq!(lines, vec!["ab", "cd", "ef"]);
    }

    #[test]
    fn resolve_export_value_maps_fields() {
        let student = build_student();
        let value = resolve_export_value("student_no", &student, 3, 2, "原因");
        match value {
            ExportValue::Text(text) => assert_eq!(text, "2023001"),
            _ => panic!("unexpected value"),
        }

        let value = resolve_export_value("approved_hours", &student, 3, 2, "原因");
        match value {
            ExportValue::Number(num) => assert_eq!(num, 2.0),
            _ => panic!("unexpected value"),
        }

        let value = resolve_export_value("reason", &student, 3, 2, "原因");
        match value {
            ExportValue::Text(text) => assert_eq!(text, "原因"),
            _ => panic!("unexpected value"),
        }

        let value = resolve_export_value("unknown", &student, 3, 2, "原因");
        match value {
            ExportValue::Text(text) => assert!(text.is_empty()),
            _ => panic!("unexpected value"),
        }
    }

    #[test]
    fn default_fields_are_ordered() {
        let summary = default_summary_fields();
        assert!(summary.windows(2).all(|pair| pair[0].order_index < pair[1].order_index));
        let student = default_student_fields();
        assert!(student.windows(2).all(|pair| pair[0].order_index < pair[1].order_index));
    }

    #[test]
    fn write_cell_accepts_text_and_number() {
        let mut workbook = rust_xlsxwriter::Workbook::new();
        let worksheet = workbook.add_worksheet();
        write_cell(worksheet, 0, 0, &ExportValue::Text("测试".to_string()))
            .expect("write text");
        write_cell(worksheet, 1, 0, &ExportValue::Number(3.0))
            .expect("write number");
    }
}
