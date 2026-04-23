use std::collections::HashSet;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::middleware::auth::AuthUser;

use super::{
    repo,
    types::{
        default_currency_code, default_currency_symbol, default_timezone, CreateUserRequest,
        FeatureFlagResponse, FeatureFlagUpdateRequest, IntakeTemplateItem,
        IntakeTemplateResponse, LocationResponse, NotificationPreferences, UpdateLocationRequest,
        UpdateUserRequest, UpdateWorkshopProfileRequest, UpsertIntakeTemplateRequest,
        UserResponse, WorkshopProfileResponse,
    },
};

const PROFILE_SETTING_KEY: &str = "settings.profile";
const NOTIFICATION_PREFERENCES_SETTING_KEY: &str = "settings.notification_preferences";
const TIMEZONE_SETTING_KEY: &str = "timezone";
const CURRENCY_CODE_SETTING_KEY: &str = "currency_code";
const CURRENCY_SYMBOL_SETTING_KEY: &str = "currency_symbol";

const SETTINGS_ADMIN_ROLES: [&str; 2] = ["OWNER", "ADMIN"];
const VALID_USER_ROLES: [&str; 7] = [
    "OWNER",
    "ADMIN",
    "MANAGER",
    "ACCOUNT_MGR",
    "MECHANIC",
    "CASHIER",
    "HR_OFFICER",
];

pub async fn get_profile(pool: &PgPool, auth: &AuthUser) -> AppResult<WorkshopProfileResponse> {
    ensure_settings_admin(auth)?;

    let mut profile = default_profile();

    if let Some(value) = repo::get_setting(pool, PROFILE_SETTING_KEY).await? {
        if let Ok(stored) = serde_json::from_str::<WorkshopProfileResponse>(&value) {
            profile = stored;
        }
    }

    if let Some(value) = repo::get_setting(pool, TIMEZONE_SETTING_KEY).await? {
        let normalized = value.trim();
        if !normalized.is_empty() {
            profile.timezone = normalized.to_string();
        }
    }

    if let Some(value) = repo::get_setting(pool, CURRENCY_CODE_SETTING_KEY).await? {
        let normalized = value.trim();
        if !normalized.is_empty() {
            profile.currency_code = normalized.to_string();
        }
    }

    if let Some(value) = repo::get_setting(pool, CURRENCY_SYMBOL_SETTING_KEY).await? {
        let normalized = value.trim();
        if !normalized.is_empty() {
            profile.currency_symbol = normalized.to_string();
        }
    }

    Ok(profile)
}

pub async fn update_profile(
    pool: &PgPool,
    auth: &AuthUser,
    req: &UpdateWorkshopProfileRequest,
) -> AppResult<WorkshopProfileResponse> {
    ensure_settings_admin(auth)?;

    let profile = normalize_profile(req)?;
    let serialized = serde_json::to_string(&profile)
        .map_err(|err| AppError::Internal(format!("Failed to serialize profile setting: {err}")))?;

    repo::upsert_setting(pool, PROFILE_SETTING_KEY, &serialized, false).await?;
    repo::upsert_setting(pool, TIMEZONE_SETTING_KEY, &profile.timezone, false).await?;
    repo::upsert_setting(pool, CURRENCY_CODE_SETTING_KEY, &profile.currency_code, false).await?;
    repo::upsert_setting(pool, CURRENCY_SYMBOL_SETTING_KEY, &profile.currency_symbol, false).await?;

    Ok(profile)
}

pub async fn list_locations(pool: &PgPool, auth: &AuthUser) -> AppResult<Vec<LocationResponse>> {
    ensure_settings_admin(auth)?;

    Ok(repo::list_locations(pool)
        .await?
        .into_iter()
        .map(LocationResponse::from)
        .collect::<Vec<_>>())
}

pub async fn create_location(
    pool: &PgPool,
    auth: &AuthUser,
    req: &UpdateLocationRequest,
) -> AppResult<LocationResponse> {
    ensure_settings_admin(auth)?;

    let name = normalize_required_text(&req.name, "Location name")?;
    let address = normalize_optional_text(req.address.as_deref());
    if req.is_primary && !req.is_active {
        return Err(AppError::Validation(
            "A primary location must be active".into(),
        ));
    }

    let has_primary = repo::count_active_primary_locations(pool, None).await? > 0;
    if !req.is_active && !has_primary {
        return Err(AppError::Conflict(
            "Create an active primary location before adding inactive locations".into(),
        ));
    }

    let should_be_primary = if req.is_active {
        req.is_primary || !has_primary
    } else {
        false
    };

    if should_be_primary {
        repo::clear_primary_locations(pool, None).await?;
    }

    let location_id = Uuid::now_v7().to_string();
    repo::create_location(
        pool,
        &location_id,
        &name,
        address.as_deref(),
        should_be_primary,
        req.is_active,
    )
    .await
    .map(LocationResponse::from)
}

pub async fn update_location(
    pool: &PgPool,
    auth: &AuthUser,
    id: &str,
    req: &UpdateLocationRequest,
) -> AppResult<LocationResponse> {
    ensure_settings_admin(auth)?;
    ensure_valid_uuid(id, "location")?;

    let existing = repo::find_location_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Location not found".into()))?;

    let name = normalize_required_text(&req.name, "Location name")?;
    let address = normalize_optional_text(req.address.as_deref());
    if req.is_primary && !req.is_active {
        return Err(AppError::Validation(
            "A primary location must be active".into(),
        ));
    }

    if existing.is_primary && (!req.is_primary || !req.is_active) {
        let other_primary_count = repo::count_active_primary_locations(pool, Some(id)).await?;
        if other_primary_count == 0 {
            return Err(AppError::Conflict(
                "At least one active primary location is required".into(),
            ));
        }
    }

    if req.is_primary {
        repo::clear_primary_locations(pool, Some(id)).await?;
    }

    repo::update_location(
        pool,
        id,
        &name,
        address.as_deref(),
        req.is_primary,
        req.is_active,
    )
    .await?
    .map(LocationResponse::from)
    .ok_or_else(|| AppError::NotFound("Location not found".into()))
}

pub async fn list_users(pool: &PgPool, auth: &AuthUser) -> AppResult<Vec<UserResponse>> {
    ensure_settings_admin(auth)?;

    Ok(repo::list_users(pool)
        .await?
        .into_iter()
        .map(UserResponse::from)
        .collect::<Vec<_>>())
}

pub async fn create_user(
    pool: &PgPool,
    auth: &AuthUser,
    req: &CreateUserRequest,
) -> AppResult<UserResponse> {
    ensure_settings_admin(auth)?;

    let email = normalize_email(&req.email)?;
    let name = normalize_required_text(&req.name, "Name")?;
    let role = normalize_user_role(&req.role)?;
    let password = normalize_password(&req.password)?;
    let password_hash = hash_password(&password)?;
    let user_id = Uuid::now_v7().to_string();

    repo::create_user(
        pool,
        &user_id,
        &email,
        &password_hash,
        &name,
        &role,
        req.is_active,
    )
    .await
    .map(UserResponse::from)
}

pub async fn update_user(
    pool: &PgPool,
    auth: &AuthUser,
    id: &str,
    req: &UpdateUserRequest,
) -> AppResult<UserResponse> {
    ensure_settings_admin(auth)?;
    ensure_valid_uuid(id, "user")?;

    let existing = repo::find_user_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    if auth.user_id == id && !req.is_active {
        return Err(AppError::Conflict(
            "You cannot deactivate your own account".into(),
        ));
    }

    let email = normalize_email(&req.email)?;
    let name = normalize_required_text(&req.name, "Name")?;
    let role = normalize_user_role(&req.role)?;

    if existing.role == "OWNER" && existing.is_active && (role != "OWNER" || !req.is_active) {
        let other_active_owners = repo::count_active_owners(pool, Some(id)).await?;
        if other_active_owners == 0 {
            return Err(AppError::Conflict(
                "At least one active owner must remain on the tenant".into(),
            ));
        }
    }

    let password = normalize_optional_password(req.password.as_deref())?;
    let updated = if let Some(password) = password {
        let password_hash = hash_password(&password)?;
        repo::update_user_with_password(
            pool,
            id,
            &email,
            &password_hash,
            &name,
            &role,
            req.is_active,
        )
        .await?
    } else {
        repo::update_user(pool, id, &email, &name, &role, req.is_active).await?
    };

    updated
        .map(UserResponse::from)
        .ok_or_else(|| AppError::NotFound("User not found".into()))
}

pub async fn delete_user(
    pool: &PgPool,
    auth: &AuthUser,
    id: &str,
) -> AppResult<serde_json::Value> {
    ensure_settings_admin(auth)?;
    ensure_valid_uuid(id, "user")?;

    let existing = repo::find_user_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    if auth.user_id == id {
        return Err(AppError::Conflict(
            "You cannot deactivate your own account".into(),
        ));
    }

    if existing.role == "OWNER" && existing.is_active {
        let other_active_owners = repo::count_active_owners(pool, Some(id)).await?;
        if other_active_owners == 0 {
            return Err(AppError::Conflict(
                "At least one active owner must remain on the tenant".into(),
            ));
        }
    }

    if !existing.is_active {
        return Ok(json!({ "deleted": true }));
    }

    let rows_affected = repo::soft_delete_user(pool, id).await?;
    if rows_affected == 0 {
        return Err(AppError::NotFound("User not found".into()));
    }

    Ok(json!({ "deleted": true }))
}

pub async fn get_intake_template(
    pool: &PgPool,
    auth: &AuthUser,
) -> AppResult<IntakeTemplateResponse> {
    ensure_settings_admin(auth)?;

    let template = repo::get_active_intake_template(pool).await?;
    Ok(match template {
        Some(template) => IntakeTemplateResponse {
            id: Some(template.id),
            name: template.name,
            items: parse_template_items(template.items),
            is_active: template.is_active,
            created_at: Some(template.created_at.to_rfc3339()),
        },
        None => IntakeTemplateResponse {
            id: None,
            name: "Default intake checklist".to_string(),
            items: vec![],
            is_active: false,
            created_at: None,
        },
    })
}

pub async fn update_intake_template(
    pool: &PgPool,
    auth: &AuthUser,
    req: &UpsertIntakeTemplateRequest,
) -> AppResult<IntakeTemplateResponse> {
    ensure_settings_admin(auth)?;

    let name = normalize_required_text(&req.name, "Template name")?;
    let items = normalize_template_items(&req.items)?;
    let items_json = serde_json::to_value(&items).map_err(|err| {
        AppError::Internal(format!("Failed to serialize intake template items: {err}"))
    })?;

    let template = repo::upsert_active_intake_template(
        pool,
        &Uuid::now_v7().to_string(),
        &name,
        &items_json,
    )
    .await?;

    Ok(IntakeTemplateResponse {
        id: Some(template.id),
        name: template.name,
        items,
        is_active: template.is_active,
        created_at: Some(template.created_at.to_rfc3339()),
    })
}

pub async fn list_feature_flags(
    control_pool: &PgPool,
    tenant_id: &str,
    auth: &AuthUser,
) -> AppResult<Vec<FeatureFlagResponse>> {
    ensure_settings_admin(auth)?;
    ensure_valid_uuid(tenant_id, "tenant")?;

    Ok(repo::list_feature_flags(control_pool, tenant_id)
        .await?
        .into_iter()
        .map(FeatureFlagResponse::from)
        .collect::<Vec<_>>())
}

pub async fn update_feature_flag(
    control_pool: &PgPool,
    tenant_id: &str,
    auth: &AuthUser,
    key: &str,
    req: &FeatureFlagUpdateRequest,
) -> AppResult<FeatureFlagResponse> {
    ensure_settings_admin(auth)?;
    ensure_valid_uuid(tenant_id, "tenant")?;

    let key = normalize_required_text(key, "Feature flag key")?;
    repo::upsert_feature_flag_override(control_pool, tenant_id, &key, req.is_enabled)
        .await
        .map(FeatureFlagResponse::from)
}

pub async fn get_notification_preferences(
    pool: &PgPool,
    auth: &AuthUser,
) -> AppResult<NotificationPreferences> {
    ensure_settings_admin(auth)?;

    let Some(value) = repo::get_setting(pool, NOTIFICATION_PREFERENCES_SETTING_KEY).await? else {
        return Ok(NotificationPreferences::default());
    };

    Ok(serde_json::from_str::<NotificationPreferences>(&value).unwrap_or_default())
}

pub async fn update_notification_preferences(
    pool: &PgPool,
    auth: &AuthUser,
    req: &NotificationPreferences,
) -> AppResult<NotificationPreferences> {
    ensure_settings_admin(auth)?;

    let preferences = NotificationPreferences {
        daily_summary_email: normalize_optional_email(req.daily_summary_email.as_deref())?,
        ..req.clone()
    };

    let serialized = serde_json::to_string(&preferences).map_err(|err| {
        AppError::Internal(format!(
            "Failed to serialize notification preferences setting: {err}"
        ))
    })?;

    repo::upsert_setting(
        pool,
        NOTIFICATION_PREFERENCES_SETTING_KEY,
        &serialized,
        false,
    )
    .await?;

    Ok(preferences)
}

fn default_profile() -> WorkshopProfileResponse {
    WorkshopProfileResponse {
        name: None,
        address: None,
        phone: None,
        email: None,
        logo_url: None,
        tax_id: None,
        timezone: default_timezone(),
        currency_code: default_currency_code(),
        currency_symbol: default_currency_symbol(),
    }
}

fn normalize_profile(req: &UpdateWorkshopProfileRequest) -> AppResult<WorkshopProfileResponse> {
    Ok(WorkshopProfileResponse {
        name: normalize_optional_text(req.name.as_deref()),
        address: normalize_optional_text(req.address.as_deref()),
        phone: normalize_optional_text(req.phone.as_deref()),
        email: normalize_optional_email(req.email.as_deref())?,
        logo_url: normalize_optional_text(req.logo_url.as_deref()),
        tax_id: normalize_optional_text(req.tax_id.as_deref()),
        timezone: normalize_optional_text(req.timezone.as_deref())
            .unwrap_or_else(default_timezone),
        currency_code: normalize_optional_text(req.currency_code.as_deref())
            .unwrap_or_else(default_currency_code),
        currency_symbol: normalize_optional_text(req.currency_symbol.as_deref())
            .unwrap_or_else(default_currency_symbol),
    })
}

fn normalize_template_items(items: &[IntakeTemplateItem]) -> AppResult<Vec<IntakeTemplateItem>> {
    let mut normalized_items = Vec::with_capacity(items.len());
    let mut seen_keys = HashSet::new();

    for item in items {
        let key = normalize_required_text(&item.key, "Template item key")?;
        if !seen_keys.insert(key.clone()) {
            return Err(AppError::Validation(
                "Template item keys must be unique".into(),
            ));
        }

        let label = normalize_required_text(&item.label, "Template item label")?;
        let input_type = normalize_required_text(&item.input_type, "Template item inputType")?;
        let options = item.options.as_ref().map(|values| {
            values
                .iter()
                .filter_map(|value| {
                    let normalized = value.trim();
                    if normalized.is_empty() {
                        None
                    } else {
                        Some(normalized.to_string())
                    }
                })
                .collect::<Vec<_>>()
        });

        normalized_items.push(IntakeTemplateItem {
            key,
            label,
            input_type,
            required: item.required,
            options: options.filter(|values| !values.is_empty()),
        });
    }

    Ok(normalized_items)
}

fn parse_template_items(value: serde_json::Value) -> Vec<IntakeTemplateItem> {
    serde_json::from_value::<Vec<IntakeTemplateItem>>(value).unwrap_or_default()
}

fn ensure_settings_admin(auth: &AuthUser) -> AppResult<()> {
    if SETTINGS_ADMIN_ROLES.contains(&auth.role.as_str()) {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "Owner or admin access is required for tenant settings".into(),
        ))
    }
}

fn normalize_required_text(value: &str, field_name: &str) -> AppResult<String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(AppError::Validation(format!("{} is required", field_name)));
    }

    Ok(normalized.to_string())
}

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value.and_then(|value| {
        let normalized = value.trim();
        if normalized.is_empty() {
            None
        } else {
            Some(normalized.to_string())
        }
    })
}

fn normalize_email(value: &str) -> AppResult<String> {
    let normalized = normalize_required_text(value, "Email")?;
    Ok(normalized.to_lowercase())
}

fn normalize_optional_email(value: Option<&str>) -> AppResult<Option<String>> {
    Ok(value.and_then(|value| {
        let normalized = value.trim();
        if normalized.is_empty() {
            None
        } else {
            Some(normalized.to_lowercase())
        }
    }))
}

fn normalize_password(value: &str) -> AppResult<String> {
    let normalized = value.trim();
    if normalized.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".into(),
        ));
    }

    Ok(normalized.to_string())
}

fn normalize_optional_password(value: Option<&str>) -> AppResult<Option<String>> {
    match value {
        Some(value) if !value.trim().is_empty() => normalize_password(value).map(Some),
        _ => Ok(None),
    }
}

fn normalize_user_role(value: &str) -> AppResult<String> {
    let normalized = normalize_required_text(value, "Role")?.to_uppercase();
    if VALID_USER_ROLES.contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(AppError::Validation(
            "Role must be one of OWNER, ADMIN, MANAGER, ACCOUNT_MGR, MECHANIC, CASHIER, or HR_OFFICER".into(),
        ))
    }
}

fn ensure_valid_uuid(value: &str, resource_name: &str) -> AppResult<()> {
    Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| AppError::Validation(format!("Invalid {} identifier", resource_name)))
}

fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| AppError::Argon2(err.to_string()))
}
