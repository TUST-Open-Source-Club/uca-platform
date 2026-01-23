//! 导出模板（Excel）占位符解析与替换。

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;

use umya_spreadsheet::structs::drawing::spreadsheet::MarkerType;
use umya_spreadsheet::structs::{Image, OrientationValues};
use umya_spreadsheet::Spreadsheet;

use crate::error::AppError;

#[derive(Debug, Clone)]
struct ListAnchor {
    sheet: String,
    column: u32,
    row: u32,
    field_key: String,
    terminator_row: Option<u32>,
}

/// 校验导出模板（Excel）占位符是否合法。
pub fn validate_export_template_bytes(bytes: &[u8]) -> Result<Vec<String>, AppError> {
    let workbook = load_workbook_from_bytes(bytes)?;
    Ok(validate_workbook(&workbook))
}

/// 根据占位符替换 Excel 模板并写出新的 xlsx。
pub fn render_template_to_xlsx(
    template_path: &Path,
    output_path: &Path,
    single_values: &HashMap<String, String>,
    list_values: &[HashMap<String, String>],
    orientation: OrientationValues,
) -> Result<(), AppError> {
    let mut workbook = umya_spreadsheet::reader::xlsx::read(template_path)
        .map_err(|_| AppError::bad_request("invalid export template"))?;

    apply_page_setup(&mut workbook, orientation);
    apply_list_placeholders(&mut workbook, list_values)?;
    apply_single_placeholders(&mut workbook, single_values)?;

    umya_spreadsheet::writer::xlsx::write(&workbook, output_path)
        .map_err(|_| AppError::internal("write export template failed"))?;
    Ok(())
}

fn load_workbook_from_bytes(bytes: &[u8]) -> Result<Spreadsheet, AppError> {
    let mut temp = tempfile::NamedTempFile::new()
        .map_err(|_| AppError::internal("create temp file failed"))?;
    temp.write_all(bytes)
        .map_err(|_| AppError::internal("write temp file failed"))?;
    umya_spreadsheet::reader::xlsx::read(temp.path())
        .map_err(|_| AppError::bad_request("invalid xlsx file"))
}

fn validate_workbook(workbook: &Spreadsheet) -> Vec<String> {
    let mut issues = Vec::new();
    let allowed_single = allowed_single_placeholders();
    let allowed_list = allowed_list_placeholders();
    let mut list_anchors = Vec::new();
    let mut terminators = Vec::new();
    let mut list_columns: HashSet<(String, u32)> = HashSet::new();

    for sheet in workbook.get_sheet_collection() {
        let sheet_name = sheet.get_name().to_string();
        for cell in iter_cells(sheet) {
            let coord = cell.coord.clone();
            let value = cell.value.clone();
            let trimmed = value.trim();
            if trimmed.starts_with("{{list:") && trimmed.ends_with("}}") {
                let field_key = trimmed.trim_start_matches("{{list:").trim_end_matches("}}").trim();
                if !is_allowed_list_field(&allowed_list, field_key) {
                    issues.push(format!(
                        "工作表 {} 单元格 {} 使用了未知列表字段 {}",
                        sheet_name, coord, field_key
                    ));
                }
                if !list_columns.insert((sheet_name.clone(), cell.column)) {
                    issues.push(format!(
                        "工作表 {} 的列 {} 存在多个列表占位符",
                        sheet_name,
                        column_label(cell.column)
                    ));
                }
                list_anchors.push(ListAnchor {
                    sheet: sheet_name.clone(),
                    column: cell.column,
                    row: cell.row,
                    field_key: field_key.to_string(),
                    terminator_row: None,
                });
                continue;
            }
            if trimmed == "{{/list}}" {
                terminators.push((sheet_name.clone(), cell.column, cell.row));
                continue;
            }

            for placeholder in extract_placeholders(value.as_str()) {
                if placeholder.starts_with("list:") || placeholder == "/list" {
                    continue;
                }
                if !allowed_single.contains(&placeholder) {
                    issues.push(format!(
                        "工作表 {} 单元格 {} 使用了未知字段 {}",
                        sheet_name, coord, placeholder
                    ));
                }
            }
        }
    }

    for (sheet, column, row) in &terminators {
        let has_anchor = list_anchors.iter().any(|anchor| {
            anchor.sheet == *sheet && anchor.column == *column && anchor.row < *row
        });
        if !has_anchor {
            issues.push(format!(
                "工作表 {} 单元格 {} 的终止符前没有列表占位符",
                sheet,
                format!("{}{}", column_label(*column), row)
            ));
        }
    }

    issues
}

fn apply_list_placeholders(
    workbook: &mut Spreadsheet,
    list_values: &[HashMap<String, String>],
) -> Result<(), AppError> {
    let anchors = collect_list_anchors(workbook);
    for anchor in anchors {
        let Some(sheet) = workbook.get_sheet_by_name_mut(&anchor.sheet) else {
            continue;
        };
        let max_len = list_values.len();
        let limit = anchor
            .terminator_row
            .and_then(|row| row.checked_sub(anchor.row))
            .map(|value| value as usize)
            .unwrap_or(max_len);
        let count = std::cmp::min(max_len, limit);
        if count == 0 {
            set_cell_value(sheet, anchor.column, anchor.row, "");
        }

        for idx in 0..count {
            let row = anchor.row + idx as u32;
            let value = if anchor.field_key == "seq" {
                (idx + 1).to_string()
            } else {
                list_values
                    .get(idx)
                    .and_then(|map| map.get(anchor.field_key.as_str()))
                    .cloned()
                    .unwrap_or_default()
            };
            set_cell_value(sheet, anchor.column, row, &value);
        }

        if let Some(terminator_row) = anchor.terminator_row {
            set_cell_value(sheet, anchor.column, terminator_row, "");
        }
    }
    Ok(())
}

fn apply_single_placeholders(
    workbook: &mut Spreadsheet,
    single_values: &HashMap<String, String>,
) -> Result<(), AppError> {
    let sheet_names: Vec<String> = workbook
        .get_sheet_collection()
        .iter()
        .map(|sheet| sheet.get_name().to_string())
        .collect();
    for sheet_name in sheet_names {
        let Some(sheet) = workbook.get_sheet_by_name_mut(&sheet_name) else {
            continue;
        };
        let cells = iter_cells(sheet);
        for cell in cells {
            let value = cell.value.clone();
            let trimmed = value.trim();
            if trimmed.starts_with("{{list:") || trimmed == "{{/list}}" {
                continue;
            }
            let mut updated = value.clone();
            let placeholders = extract_placeholders(value.as_str());
            for placeholder in placeholders {
                if placeholder.starts_with("list:") || placeholder == "/list" {
                    continue;
                }
                if placeholder == "first_signature_image" || placeholder == "final_signature_image" {
                    let path = single_values.get(&placeholder).cloned().unwrap_or_default();
                    insert_signature_image(sheet, cell.column, cell.row, &path)?;
                    let token = format!("{{{{{placeholder}}}}}");
                    updated = updated.replace(&token, "");
                } else {
                    let replace = single_values
                        .get(&placeholder)
                        .cloned()
                        .unwrap_or_default();
                    let token = format!("{{{{{placeholder}}}}}");
                    updated = updated.replace(&token, &replace);
                }
            }
            if updated != value {
                set_cell_value(sheet, cell.column, cell.row, &updated);
            }
        }
    }
    Ok(())
}

fn collect_list_anchors(workbook: &Spreadsheet) -> Vec<ListAnchor> {
    let mut anchors = Vec::new();
    let mut terminators: HashMap<(String, u32), Vec<u32>> = HashMap::new();

    for sheet in workbook.get_sheet_collection() {
        let sheet_name = sheet.get_name().to_string();
        for cell in iter_cells(sheet) {
            let trimmed = cell.value.trim();
            if trimmed.starts_with("{{list:") && trimmed.ends_with("}}") {
                let field_key = trimmed
                    .trim_start_matches("{{list:")
                    .trim_end_matches("}}")
                    .trim()
                    .to_string();
                anchors.push(ListAnchor {
                    sheet: sheet_name.clone(),
                    column: cell.column,
                    row: cell.row,
                    field_key,
                    terminator_row: None,
                });
            } else if trimmed == "{{/list}}" {
                terminators
                    .entry((sheet_name.clone(), cell.column))
                    .or_default()
                    .push(cell.row);
            }
        }
    }

    for anchor in &mut anchors {
        if let Some(rows) = terminators.get(&(anchor.sheet.clone(), anchor.column)) {
            let mut candidates = rows.iter().filter(|row| **row > anchor.row).cloned().collect::<Vec<_>>();
            candidates.sort_unstable();
            anchor.terminator_row = candidates.first().copied();
        }
    }

    anchors
}

struct CellInfo {
    coord: String,
    column: u32,
    row: u32,
    value: String,
}

fn iter_cells(sheet: &umya_spreadsheet::Worksheet) -> Vec<CellInfo> {
    let mut items = Vec::new();
    for cell in sheet.get_cell_collection() {
        let coord = cell.get_coordinate().get_coordinate();
        let column = *cell.get_coordinate().get_col_num();
        let row = *cell.get_coordinate().get_row_num();
        let value = cell.get_value().to_string();
        if !value.trim().is_empty() {
            items.push(CellInfo {
                coord,
                column,
                row,
                value,
            });
        }
    }
    items
}

fn set_cell_value(sheet: &mut umya_spreadsheet::Worksheet, column: u32, row: u32, value: &str) {
    let coord = format!("{}{}", column_label(column), row);
    let cell = sheet.get_cell_mut(coord.as_str());
    cell.set_value(value);
    adjust_cell_size(sheet, column, row, value);
}

fn insert_signature_image(
    sheet: &mut umya_spreadsheet::Worksheet,
    column: u32,
    row: u32,
    path: &str,
) -> Result<(), AppError> {
    if path.trim().is_empty() {
        return Ok(());
    }
    let path_obj = std::path::Path::new(path);
    if !path_obj.exists() {
        return Ok(());
    }
    let coord = format!("{}{}", column_label(column), row);
    let mut marker = MarkerType::default();
    marker.set_coordinate(coord.as_str());
    let mut image = Image::default();
    image.new_image(path, marker);
    if let Some(anchor) = image.get_one_cell_anchor_mut() {
        let (cx, cy) = cell_extent_emu(sheet, column, row);
        if cx > 0 && cy > 0 {
            anchor.get_extent_mut().set_cx(cx);
            anchor.get_extent_mut().set_cy(cy);
        }
    }
    sheet.add_image(image);
    Ok(())
}

fn apply_page_setup(workbook: &mut Spreadsheet, orientation: OrientationValues) {
    for sheet in workbook.get_sheet_collection_mut() {
        let setup = sheet.get_page_setup_mut();
        setup.set_paper_size(9);
        setup.set_orientation(orientation.clone());
    }
}

fn adjust_cell_size(
    sheet: &mut umya_spreadsheet::Worksheet,
    column: u32,
    row: u32,
    value: &str,
) {
    if value.trim().is_empty() {
        return;
    }
    let text_len = value.chars().count().max(1);
    let estimated_width = (text_len as f64) * 1.2;
    let column_dimension = sheet.get_column_dimension_by_number_mut(&column);
    let current_width = *column_dimension.get_width();
    if estimated_width > current_width {
        column_dimension.set_width(estimated_width);
    }

    let max_chars_per_line = ((current_width.max(8.38) * 1.1).floor() as usize).max(1);
    let lines = (text_len + max_chars_per_line - 1) / max_chars_per_line;
    let target_height = 15.0 * lines as f64;
    let row_dimension = sheet.get_row_dimension_mut(&row);
    let current_height = *row_dimension.get_height();
    if target_height > current_height {
        row_dimension.set_height(target_height);
    }
}

fn cell_extent_emu(
    sheet: &umya_spreadsheet::Worksheet,
    column: u32,
    row: u32,
) -> (i64, i64) {
    let col_width = sheet
        .get_column_dimension_by_number(&column)
        .map(|col| *col.get_width())
        .unwrap_or(8.38);
    let row_height = sheet
        .get_row_dimension(&row)
        .map(|row| *row.get_height())
        .unwrap_or(15.0);
    let px_width = (col_width * 7.0 + 5.0).max(1.0);
    let px_height = (row_height * 96.0 / 72.0).max(1.0);
    let cx = (px_width * 9525.0) as i64;
    let cy = (px_height * 9525.0) as i64;
    (cx, cy)
}

fn extract_placeholders(value: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut start = 0usize;
    while let Some(open) = value[start..].find("{{") {
        let open_idx = start + open + 2;
        if let Some(close) = value[open_idx..].find("}}") {
            let close_idx = open_idx + close;
            let token = value[open_idx..close_idx].trim();
            if !token.is_empty() {
                placeholders.push(token.to_string());
            }
            start = close_idx + 2;
        } else {
            break;
        }
    }
    placeholders
}

fn column_label(mut column: u32) -> String {
    let mut label = String::new();
    while column > 0 {
        let rem = ((column - 1) % 26) as u8;
        label.insert(0, (b'A' + rem) as char);
        column = (column - 1) / 26;
    }
    label
}

fn allowed_single_placeholders() -> HashSet<String> {
    [
        "student_no",
        "name",
        "gender",
        "department",
        "major",
        "class_name",
        "phone",
        "total_self_hours",
        "total_approved_hours",
        "total_reason",
        "first_signature_path",
        "final_signature_path",
        "first_signature_image",
        "final_signature_image",
    ]
    .iter()
    .map(|value| value.to_string())
    .collect()
}

fn allowed_list_placeholders() -> HashSet<String> {
    [
        "seq",
        "contest_year",
        "contest_category",
        "contest_name",
        "contest_level",
        "contest_role",
        "award_level",
        "award_date",
        "self_hours",
        "first_review_hours",
        "final_review_hours",
        "approved_hours",
        "recommended_hours",
        "status",
        "rejection_reason",
    ]
    .iter()
    .map(|value| value.to_string())
    .collect()
}

fn is_allowed_list_field(allowed: &HashSet<String>, field_key: &str) -> bool {
    if allowed.contains(field_key) {
        return true;
    }
    field_key.starts_with("custom.")
}

#[cfg(test)]
mod tests {
    use super::{allowed_list_placeholders, allowed_single_placeholders};

    #[test]
    fn list_placeholders_include_seq() {
        let allowed = allowed_list_placeholders();
        assert!(allowed.contains("seq"));
    }

    #[test]
    fn single_placeholders_include_signature_images() {
        let allowed = allowed_single_placeholders();
        assert!(allowed.contains("first_signature_image"));
        assert!(allowed.contains("final_signature_image"));
    }
}
