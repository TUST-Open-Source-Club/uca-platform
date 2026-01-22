//! 劳动教育学时规则与计算。

use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use crate::{
    entities::{labor_hour_rules, LaborHourRule},
    error::AppError,
    state::AppState,
};

#[derive(Debug, Clone, Copy)]
pub struct LaborHourRuleConfig {
    pub base_hours_a: i32,
    pub base_hours_b: i32,
    pub national_leader_hours: i32,
    pub national_member_hours: i32,
    pub provincial_leader_hours: i32,
    pub provincial_member_hours: i32,
    pub school_leader_hours: i32,
    pub school_member_hours: i32,
}

impl Default for LaborHourRuleConfig {
    fn default() -> Self {
        Self {
            base_hours_a: 2,
            base_hours_b: 2,
            national_leader_hours: 4,
            national_member_hours: 2,
            provincial_leader_hours: 2,
            provincial_member_hours: 1,
            school_leader_hours: 1,
            school_member_hours: 1,
        }
    }
}

pub async fn load_labor_hour_rules(state: &AppState) -> Result<LaborHourRuleConfig, AppError> {
    if let Some(rule) = LaborHourRule::find()
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
    {
        return Ok(LaborHourRuleConfig {
            base_hours_a: rule.base_hours_a,
            base_hours_b: rule.base_hours_b,
            national_leader_hours: rule.national_leader_hours,
            national_member_hours: rule.national_member_hours,
            provincial_leader_hours: rule.provincial_leader_hours,
            provincial_member_hours: rule.provincial_member_hours,
            school_leader_hours: rule.school_leader_hours,
            school_member_hours: rule.school_member_hours,
        });
    }
    Ok(LaborHourRuleConfig::default())
}

pub async fn upsert_labor_hour_rules(
    state: &AppState,
    config: LaborHourRuleConfig,
) -> Result<LaborHourRuleConfig, AppError> {
    let now = Utc::now();
    if let Some(existing) = LaborHourRule::find()
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?
    {
        let mut active: labor_hour_rules::ActiveModel = existing.into();
        active.base_hours_a = Set(config.base_hours_a);
        active.base_hours_b = Set(config.base_hours_b);
        active.national_leader_hours = Set(config.national_leader_hours);
        active.national_member_hours = Set(config.national_member_hours);
        active.provincial_leader_hours = Set(config.provincial_leader_hours);
        active.provincial_member_hours = Set(config.provincial_member_hours);
        active.school_leader_hours = Set(config.school_leader_hours);
        active.school_member_hours = Set(config.school_member_hours);
        active.updated_at = Set(now);
        active
            .update(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    } else {
        let model = labor_hour_rules::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            base_hours_a: Set(config.base_hours_a),
            base_hours_b: Set(config.base_hours_b),
            national_leader_hours: Set(config.national_leader_hours),
            national_member_hours: Set(config.national_member_hours),
            provincial_leader_hours: Set(config.provincial_leader_hours),
            provincial_member_hours: Set(config.provincial_member_hours),
            school_leader_hours: Set(config.school_leader_hours),
            school_member_hours: Set(config.school_member_hours),
            created_at: Set(now),
            updated_at: Set(now),
        };
        labor_hour_rules::Entity::insert(model)
            .exec_without_returning(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    }
    Ok(config)
}

pub fn compute_recommended_hours(
    config: LaborHourRuleConfig,
    category: Option<&str>,
    level: Option<&str>,
    role: Option<&str>,
) -> i32 {
    let mut hours = 0;
    if let Some(value) = category {
        let normalized = value.trim().to_uppercase();
        if normalized == "A" {
            hours += config.base_hours_a;
        } else if normalized == "B" {
            hours += config.base_hours_b;
        }
    }

    let level_norm = level.unwrap_or("").trim();
    let role_norm = role.unwrap_or("").trim();
    match (level_norm, role_norm) {
        ("国家级", "负责人") | ("国家级", "leader") | ("national", "leader") => {
            hours += config.national_leader_hours
        }
        ("国家级", "成员") | ("国家级", "member") | ("national", "member") => {
            hours += config.national_member_hours
        }
        ("省级", "负责人") | ("省级", "leader") | ("provincial", "leader") => {
            hours += config.provincial_leader_hours
        }
        ("省级", "成员") | ("省级", "member") | ("provincial", "member") => {
            hours += config.provincial_member_hours
        }
        ("校级", "负责人") | ("校级", "leader") | ("school", "leader") => {
            hours += config.school_leader_hours
        }
        ("校级", "成员") | ("校级", "member") | ("school", "member") => {
            hours += config.school_member_hours
        }
        _ => {}
    }

    hours
}
