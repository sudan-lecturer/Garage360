use axum::{
    extract::{Path, State},
    routing::{delete as delete_route, get, post, put},
    Json, Router,
};
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::middleware::{auth::AuthUser, tenant::TenantDbPool};
use crate::AppState;

use super::{
    service,
    types::{
        CreateUserRequest, FeatureFlagUpdateRequest, NotificationPreferences,
        UpdateLocationRequest, UpdateUserRequest, UpdateWorkshopProfileRequest,
        UpsertIntakeTemplateRequest,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/settings/profile", get(get_profile))
        .route("/settings/profile", put(update_profile))
        .route("/settings/locations", get(list_locations))
        .route("/settings/locations", post(create_location))
        .route("/settings/locations/:id", put(update_location))
        .route("/settings/users", get(list_users))
        .route("/settings/users", post(create_user))
        .route("/settings/users/:id", put(update_user))
        .route("/settings/users/:id", delete_route(delete_user))
        .route("/settings/intake-template", get(get_intake_template))
        .route("/settings/intake-template", put(update_intake_template))
        .route("/settings/feature-flags", get(list_feature_flags))
        .route("/settings/feature-flags/:key", put(update_feature_flag))
        .route(
            "/settings/notification-preferences",
            get(get_notification_preferences),
        )
        .route(
            "/settings/notification-preferences",
            put(update_notification_preferences),
        )
}

async fn get_profile(
    tenant_db: TenantDbPool,
    auth: AuthUser,
) -> AppResult<Json<super::types::WorkshopProfileResponse>> {
    Ok(Json(service::get_profile(&tenant_db.pool, &auth).await?))
}

async fn update_profile(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Json(req): Json<UpdateWorkshopProfileRequest>,
) -> AppResult<Json<super::types::WorkshopProfileResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::update_profile(&tenant_db.pool, &auth, &req).await?,
    ))
}

async fn list_locations(
    tenant_db: TenantDbPool,
    auth: AuthUser,
) -> AppResult<Json<Vec<super::types::LocationResponse>>> {
    Ok(Json(service::list_locations(&tenant_db.pool, &auth).await?))
}

async fn create_location(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Json(req): Json<UpdateLocationRequest>,
) -> AppResult<Json<super::types::LocationResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::create_location(&tenant_db.pool, &auth, &req).await?,
    ))
}

async fn update_location(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<UpdateLocationRequest>,
) -> AppResult<Json<super::types::LocationResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::update_location(&tenant_db.pool, &auth, &id, &req).await?,
    ))
}

async fn list_users(
    tenant_db: TenantDbPool,
    auth: AuthUser,
) -> AppResult<Json<Vec<super::types::UserResponse>>> {
    Ok(Json(service::list_users(&tenant_db.pool, &auth).await?))
}

async fn create_user(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<super::types::UserResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(service::create_user(&tenant_db.pool, &auth, &req).await?))
}

async fn update_user(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<Json<super::types::UserResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::update_user(&tenant_db.pool, &auth, &id, &req).await?,
    ))
}

async fn delete_user(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(service::delete_user(&tenant_db.pool, &auth, &id).await?))
}

async fn get_intake_template(
    tenant_db: TenantDbPool,
    auth: AuthUser,
) -> AppResult<Json<super::types::IntakeTemplateResponse>> {
    Ok(Json(
        service::get_intake_template(&tenant_db.pool, &auth).await?,
    ))
}

async fn update_intake_template(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Json(req): Json<UpsertIntakeTemplateRequest>,
) -> AppResult<Json<super::types::IntakeTemplateResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::update_intake_template(&tenant_db.pool, &auth, &req).await?,
    ))
}

async fn list_feature_flags(
    State(state): State<AppState>,
    tenant_db: TenantDbPool,
    auth: AuthUser,
) -> AppResult<Json<Vec<super::types::FeatureFlagResponse>>> {
    Ok(Json(
        service::list_feature_flags(&state.control_db, &tenant_db.tenant_id, &auth).await?,
    ))
}

async fn update_feature_flag(
    State(state): State<AppState>,
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(key): Path<String>,
    Json(req): Json<FeatureFlagUpdateRequest>,
) -> AppResult<Json<super::types::FeatureFlagResponse>> {
    Ok(Json(
        service::update_feature_flag(&state.control_db, &tenant_db.tenant_id, &auth, &key, &req)
            .await?,
    ))
}

async fn get_notification_preferences(
    tenant_db: TenantDbPool,
    auth: AuthUser,
) -> AppResult<Json<NotificationPreferences>> {
    Ok(Json(
        service::get_notification_preferences(&tenant_db.pool, &auth).await?,
    ))
}

async fn update_notification_preferences(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Json(req): Json<NotificationPreferences>,
) -> AppResult<Json<NotificationPreferences>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::update_notification_preferences(&tenant_db.pool, &auth, &req).await?,
    ))
}
