//! 密码策略读取与更新。

use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use uuid::Uuid;

use crate::config::PasswordPolicy;
use crate::entities::{password_policies, PasswordPolicy as PasswordPolicyEntity};
use crate::error::AppError;
use crate::state::AppState;

pub async fn load_password_policy(state: &AppState) -> Result<PasswordPolicy, AppError> {
    let record = PasswordPolicyEntity::find()
        .order_by_desc(password_policies::Column::UpdatedAt)
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    if let Some(model) = record {
        return Ok(PasswordPolicy {
            min_length: model.min_length as usize,
            require_uppercase: model.require_uppercase,
            require_lowercase: model.require_lowercase,
            require_digit: model.require_digit,
            require_symbol: model.require_symbol,
        });
    }
    Ok(state.config.password_policy.clone())
}

pub async fn upsert_password_policy(
    state: &AppState,
    policy: PasswordPolicy,
) -> Result<PasswordPolicy, AppError> {
    let existing = PasswordPolicyEntity::find()
        .order_by_desc(password_policies::Column::UpdatedAt)
        .one(&state.db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    let now = Utc::now();
    if let Some(record) = existing {
        let mut active: password_policies::ActiveModel = record.into();
        active.min_length = Set(policy.min_length as i32);
        active.require_uppercase = Set(policy.require_uppercase);
        active.require_lowercase = Set(policy.require_lowercase);
        active.require_digit = Set(policy.require_digit);
        active.require_symbol = Set(policy.require_symbol);
        active.updated_at = Set(now);
        active
            .update(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    } else {
        let model = password_policies::ActiveModel {
            id: Set(Uuid::new_v4()),
            min_length: Set(policy.min_length as i32),
            require_uppercase: Set(policy.require_uppercase),
            require_lowercase: Set(policy.require_lowercase),
            require_digit: Set(policy.require_digit),
            require_symbol: Set(policy.require_symbol),
            created_at: Set(now),
            updated_at: Set(now),
        };
        PasswordPolicyEntity::insert(model)
            .exec_without_returning(&state.db)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;
    }
    Ok(policy)
}
