//! 模板配置辅助函数（导入映射等）。

use std::collections::HashMap;

use calamine::Data;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set, ActiveModelTrait};
use serde_json::Value;

use crate::{
    entities::{
        export_templates, import_template_fields, import_templates, ExportTemplate, ImportTemplate,
        ImportTemplateField,
    },
    error::AppError,
    state::AppState,
};

#[derive(Clone, Debug)]
pub struct ImportFieldConfig {
    pub field_key: String,
    pub label: String,
    pub column_title: String,
    pub required: bool,
    pub order_index: i32,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ImportTemplateConfig {
    pub template_key: String,
    pub name: String,
    pub fields: Vec<ImportFieldConfig>,
}

#[derive(Clone, Debug)]
pub struct ExportTemplateConfig {
    pub template_key: String,
    pub name: String,
    pub layout: Value,
}

/// 读取导入模板配置（不存在时返回默认模板）。
pub async fn load_import_template(
    state: &AppState,
    template_key: &str,
) -> Result<ImportTemplateConfig, AppError> {
    if let Some(template) = ImportTemplate::find()
        .filter(import_templates::Column::TemplateKey.eq(template_key))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
    {
        let fields = ImportTemplateField::find()
            .filter(import_template_fields::Column::TemplateId.eq(template.id))
            .all(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
        let mut configs = fields
            .into_iter()
            .map(|field| ImportFieldConfig {
                field_key: field.field_key,
                label: field.label,
                column_title: field.column_title,
                required: field.required,
                order_index: field.order_index,
                description: field.description,
            })
            .collect::<Vec<_>>();
        configs.sort_by_key(|field| field.order_index);
        return Ok(ImportTemplateConfig {
            template_key: template.template_key,
            name: template.name,
            fields: configs,
        });
    }

    Ok(default_import_template(template_key))
}

/// 读取导出模板配置（不存在时返回默认模板）。
pub async fn load_export_template(
    state: &AppState,
    template_key: &str,
) -> Result<ExportTemplateConfig, AppError> {
    if let Some(template) = ExportTemplate::find()
        .filter(export_templates::Column::TemplateKey.eq(template_key))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
    {
        let layout: Value = serde_json::from_str(&template.layout_json)
            .map_err(|_| AppError::internal("invalid export template"))?;
        return Ok(ExportTemplateConfig {
            template_key: template.template_key,
            name: template.name,
            layout,
        });
    }

    Ok(default_export_template(template_key))
}

/// 新增或更新导出模板配置。
pub async fn upsert_export_template(
    state: &AppState,
    template_key: &str,
    name: String,
    layout: Value,
) -> Result<ExportTemplateConfig, AppError> {
    let now = chrono::Utc::now();
    let layout_json = serde_json::to_string(&layout)
        .map_err(|_| AppError::bad_request("invalid layout json"))?;

    if let Some(existing) = ExportTemplate::find()
        .filter(export_templates::Column::TemplateKey.eq(template_key))
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
    {
        let mut active: export_templates::ActiveModel = existing.into();
        active.name = Set(name.clone());
        active.layout_json = Set(layout_json);
        active.updated_at = Set(now);
        active
            .update(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    } else {
        let model = export_templates::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            template_key: Set(template_key.to_string()),
            name: Set(name.clone()),
            layout_json: Set(layout_json),
            created_at: Set(now),
            updated_at: Set(now),
        };
        export_templates::Entity::insert(model)
            .exec_without_returning(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    }

    Ok(ExportTemplateConfig {
        template_key: template_key.to_string(),
        name,
        layout,
    })
}

/// 构建表头名称到列索引的映射。
pub fn build_header_index(header_row: Option<&[Data]>) -> HashMap<String, usize> {
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

/// 根据表头名称读取单元格内容。
pub fn read_cell_by_title(header_index: &HashMap<String, usize>, title: &str, row: &[Data]) -> String {
    if let Some(idx) = header_index.get(title) {
        return read_cell_by_index(*idx, row);
    }
    String::new()
}

/// 根据列索引读取单元格内容。
pub fn read_cell_by_index(idx: usize, row: &[Data]) -> String {
    row.get(idx)
        .map(|cell| cell.to_string().trim().to_string())
        .unwrap_or_default()
}

/// 将导入模板字段映射到表头索引。
pub fn map_import_fields(
    header_index: &HashMap<String, usize>,
    fields: &[ImportFieldConfig],
) -> Result<HashMap<String, usize>, AppError> {
    let mut map = HashMap::new();
    for field in fields {
        if field.column_title.trim().is_empty() {
            if field.required {
                return Err(AppError::bad_request("missing required header"));
            }
            continue;
        }
        if let Some(idx) = header_index.get(field.column_title.as_str()) {
            map.insert(field.field_key.clone(), *idx);
        } else if field.required {
            return Err(AppError::bad_request("missing required header"));
        }
    }
    Ok(map)
}

fn default_import_template(template_key: &str) -> ImportTemplateConfig {
    match template_key {
        "competition_library" => ImportTemplateConfig {
            template_key: template_key.to_string(),
            name: "认可竞赛列表".to_string(),
            fields: vec![
                ImportFieldConfig {
                    field_key: "contest_year".to_string(),
                    label: "年份".to_string(),
                    column_title: "年份".to_string(),
                    required: true,
                    order_index: 1,
                    description: Some("竞赛年份".to_string()),
                },
                ImportFieldConfig {
                    field_key: "contest_category".to_string(),
                    label: "竞赛类型".to_string(),
                    column_title: "竞赛类型".to_string(),
                    required: true,
                    order_index: 2,
                    description: Some("A/B 类".to_string()),
                },
                ImportFieldConfig {
                    field_key: "contest_name".to_string(),
                    label: "竞赛名称".to_string(),
                    column_title: "竞赛名称".to_string(),
                    required: true,
                    order_index: 3,
                    description: Some("标准竞赛名称".to_string()),
                },
            ],
        },
        "students" => ImportTemplateConfig {
            template_key: template_key.to_string(),
            name: "学生名单".to_string(),
            fields: vec![
                ImportFieldConfig {
                    field_key: "student_no".to_string(),
                    label: "学号".to_string(),
                    column_title: "学号".to_string(),
                    required: true,
                    order_index: 1,
                    description: None,
                },
                ImportFieldConfig {
                    field_key: "name".to_string(),
                    label: "姓名".to_string(),
                    column_title: "姓名".to_string(),
                    required: true,
                    order_index: 2,
                    description: None,
                },
                ImportFieldConfig {
                    field_key: "gender".to_string(),
                    label: "性别".to_string(),
                    column_title: "性别".to_string(),
                    required: false,
                    order_index: 3,
                    description: None,
                },
                ImportFieldConfig {
                    field_key: "department".to_string(),
                    label: "院系".to_string(),
                    column_title: "院系".to_string(),
                    required: false,
                    order_index: 4,
                    description: None,
                },
                ImportFieldConfig {
                    field_key: "major".to_string(),
                    label: "专业".to_string(),
                    column_title: "专业".to_string(),
                    required: false,
                    order_index: 5,
                    description: None,
                },
                ImportFieldConfig {
                    field_key: "class_name".to_string(),
                    label: "班级".to_string(),
                    column_title: "班级".to_string(),
                    required: false,
                    order_index: 6,
                    description: None,
                },
                ImportFieldConfig {
                    field_key: "phone".to_string(),
                    label: "手机号".to_string(),
                    column_title: "手机号".to_string(),
                    required: false,
                    order_index: 7,
                    description: None,
                },
            ],
        },
        _ => ImportTemplateConfig {
            template_key: template_key.to_string(),
            name: "学生获奖情况清单".to_string(),
            fields: vec![
                ImportFieldConfig {
                    field_key: "student_no".to_string(),
                    label: "学号".to_string(),
                    column_title: "学号".to_string(),
                    required: true,
                    order_index: 1,
                    description: None,
                },
                ImportFieldConfig {
                    field_key: "contest_name".to_string(),
                    label: "竞赛名称".to_string(),
                    column_title: "竞赛名称".to_string(),
                    required: true,
                    order_index: 2,
                    description: None,
                },
                ImportFieldConfig {
                    field_key: "contest_level".to_string(),
                    label: "竞赛级别".to_string(),
                    column_title: "竞赛级别".to_string(),
                    required: true,
                    order_index: 3,
                    description: Some("国家级/省级/校级".to_string()),
                },
                ImportFieldConfig {
                    field_key: "award_level".to_string(),
                    label: "获奖等级".to_string(),
                    column_title: "获奖等级".to_string(),
                    required: true,
                    order_index: 4,
                    description: None,
                },
                ImportFieldConfig {
                    field_key: "contest_role".to_string(),
                    label: "负责人/成员".to_string(),
                    column_title: "角色".to_string(),
                    required: true,
                    order_index: 5,
                    description: Some("负责人/成员".to_string()),
                },
                ImportFieldConfig {
                    field_key: "award_date".to_string(),
                    label: "时间".to_string(),
                    column_title: "时间".to_string(),
                    required: false,
                    order_index: 6,
                    description: Some("获奖时间".to_string()),
                },
                ImportFieldConfig {
                    field_key: "self_hours".to_string(),
                    label: "自评学时".to_string(),
                    column_title: "自评学时".to_string(),
                    required: true,
                    order_index: 7,
                    description: None,
                },
            ],
        },
    }
}

fn default_export_template(template_key: &str) -> ExportTemplateConfig {
    let layout = match template_key {
        "labor_hours" => serde_json::json!({
            "title": "劳动教育学时认定表",
            "sections": [
                {
                    "type": "info",
                    "title": "学生信息",
                    "fields": [
                        { "key": "student_no", "label": "学号" },
                        { "key": "name", "label": "姓名" },
                        { "key": "department", "label": "院系" },
                        { "key": "major", "label": "专业" },
                        { "key": "class_name", "label": "班级" }
                    ]
                },
                {
                    "type": "table",
                    "title": "竞赛获奖明细",
                    "columns": [
                        { "key": "contest_year", "label": "年份" },
                        { "key": "contest_category", "label": "竞赛类型" },
                        { "key": "contest_name", "label": "竞赛名称" },
                        { "key": "contest_level", "label": "竞赛级别" },
                        { "key": "contest_role", "label": "角色" },
                        { "key": "award_level", "label": "获奖等级" },
                        { "key": "self_hours", "label": "自评学时" },
                        { "key": "approved_hours", "label": "审核学时" }
                    ]
                }
            ],
            "signature": {
                "first_label": "初审教师签名",
                "final_label": "复审教师签名"
            }
        }),
        _ => serde_json::json!({ "title": "导出模板", "sections": [] }),
    };

    ExportTemplateConfig {
        template_key: template_key.to_string(),
        name: "劳动教育学时认定表".to_string(),
        layout,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::Data;

    #[test]
    fn build_header_index_maps_columns() {
        let header = vec![Data::String("学号".into()), Data::String("竞赛名称".into())];
        let index = build_header_index(Some(&header));
        assert_eq!(index.get("学号"), Some(&0));
        assert_eq!(index.get("竞赛名称"), Some(&1));
    }

    #[test]
    fn read_cell_by_title_uses_header_index() {
        let header = vec![Data::String("学号".into()), Data::String("竞赛名称".into())];
        let index = build_header_index(Some(&header));
        let row = vec![Data::String("2023001".into()), Data::String("竞赛A".into())];
        let value = read_cell_by_title(&index, "竞赛名称", &row);
        assert_eq!(value, "竞赛A");
    }

    #[test]
    fn map_import_fields_requires_headers() {
        let header = vec![Data::String("学号".into())];
        let index = build_header_index(Some(&header));
        let fields = vec![
            ImportFieldConfig {
                field_key: "student_no".to_string(),
                label: "学号".to_string(),
                column_title: "学号".to_string(),
                required: true,
                order_index: 1,
                description: None,
            },
            ImportFieldConfig {
                field_key: "contest_name".to_string(),
                label: "竞赛名称".to_string(),
                column_title: "竞赛名称".to_string(),
                required: true,
                order_index: 2,
                description: None,
            },
        ];
        let result = map_import_fields(&index, &fields);
        assert!(result.is_err());
    }
}
