use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::codecs::webp::WebPEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageEncoder};
use serde_json::json;
use sqlx::PgPool;
use std::io::Cursor;
use tracing::warn;

use crate::errors::{AppError, AppResult};
use crate::common::pagination::PaginationMeta;
use crate::storage::Storage;

use super::{
    repo,
    types::{VehicleRequest, VehicleResponse},
};

pub async fn list_vehicles(
    pool: &PgPool,
    page: i64,
    limit: i64,
    search: String,
) -> AppResult<serde_json::Value> {
    repo::ensure_vehicle_support_tables(pool).await?;
    let offset = (page - 1) * limit;
    let like = format!("%{}%", search);

    let vehicles = repo::list(pool, &search, &like, limit, offset).await?;
    let total = repo::count(pool, &search, &like).await?;

    Ok(json!({
        "data": vehicles,
        "meta": crate::common::pagination::PaginationMeta { page, limit, total }
    }))
}

pub async fn search_vehicles(
    pool: &PgPool,
    search: String,
) -> AppResult<Vec<VehicleResponse>> {
    repo::ensure_vehicle_support_tables(pool).await?;
    let like = format!("%{}%", search);
    repo::list(pool, &search, &like, 20, 0).await
}

pub async fn get_vehicle(pool: &PgPool, id: &str) -> AppResult<VehicleResponse> {
    repo::ensure_vehicle_support_tables(pool).await?;
    repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Vehicle not found".into()))
}

pub async fn create_vehicle(
    storage: &Storage,
    pool: &PgPool,
    req: &VehicleRequest,
    created_by: &str,
) -> AppResult<VehicleResponse> {
    repo::ensure_vehicle_support_tables(pool).await?;
    let prepared_photo = if let Some(photo_base64) = &req.photo_base64 {
        Some(prepare_vehicle_photo(photo_base64)?)
    } else {
        None
    };
    let vehicle_id = repo::create(pool, req, created_by).await?;

    if let Some(encoded) = prepared_photo {
        let path = format!("vehicles/{vehicle_id}/primary.webp");
        if let Err(err) = storage.put_bytes(&path, encoded).await {
            warn!("vehicle image upload failed for {}: {}", vehicle_id, err);
        } else if let Err(err) = repo::upsert_vehicle_photo(pool, &vehicle_id, &path, Some(created_by)).await {
            warn!("vehicle image metadata save failed for {}: {}", vehicle_id, err);
        }
    }

    get_vehicle(pool, &vehicle_id).await
}

pub async fn update_vehicle(
    storage: &Storage,
    pool: &PgPool,
    id: &str,
    req: &VehicleRequest,
) -> AppResult<VehicleResponse> {
    repo::ensure_vehicle_support_tables(pool).await?;
    let prepared_photo = if let Some(photo_base64) = &req.photo_base64 {
        Some(prepare_vehicle_photo(photo_base64)?)
    } else {
        None
    };
    let vehicle = repo::update(pool, id, req)
        .await?
        .ok_or_else(|| AppError::NotFound("Vehicle not found".into()))?;

    if let Some(encoded) = prepared_photo {
        let path = format!("vehicles/{id}/primary.webp");
        if let Err(err) = storage.put_bytes(&path, encoded).await {
            warn!("vehicle image upload failed for {}: {}", id, err);
        } else if let Err(err) = repo::upsert_vehicle_photo(pool, id, &path, None).await {
            warn!("vehicle image metadata save failed for {}: {}", id, err);
        }
    }

    Ok(vehicle)
}

pub async fn delete_vehicle(pool: &PgPool, id: &str) -> AppResult<serde_json::Value> {
    repo::ensure_vehicle_support_tables(pool).await?;
    let rows_affected = repo::soft_delete(pool, id).await?;
    if rows_affected == 0 {
        return Err(AppError::NotFound("Vehicle not found".into()));
    }

    Ok(json!({ "deleted": true }))
}

fn decode_base64_image(input: &str) -> AppResult<Vec<u8>> {
    let payload = input
        .split(',')
        .next_back()
        .ok_or_else(|| AppError::Validation("Invalid image payload".into()))?;
    STANDARD
        .decode(payload)
        .map_err(|_| AppError::Validation("Invalid base64 encoding".into()))
}

fn prepare_vehicle_photo(photo_base64: &str) -> AppResult<Vec<u8>> {
    let bytes = decode_base64_image(photo_base64)?;
    let image = image::load_from_memory(&bytes)
        .map_err(|_| AppError::Validation("Invalid vehicle image payload".into()))?;
    let normalized = normalize_vehicle_photo(image);
    encode_webp(&normalized)
}

fn normalize_vehicle_photo(image: DynamicImage) -> DynamicImage {
    let (w, h) = image.dimensions();
    if w <= 1600 && h <= 1600 {
        image
    } else if w >= h {
        image.resize(1600, ((h as f32 / w as f32) * 1600.0).round() as u32, FilterType::Lanczos3)
    } else {
        image.resize(((w as f32 / h as f32) * 1600.0).round() as u32, 1600, FilterType::Lanczos3)
    }
}

fn encode_webp(image: &DynamicImage) -> AppResult<Vec<u8>> {
    let mut output = Cursor::new(Vec::new());
    let rgba = image.to_rgba8();
    let encoder = WebPEncoder::new_lossless(&mut output);
    encoder
        .write_image(&rgba, rgba.width(), rgba.height(), image::ExtendedColorType::Rgba8)
        .map_err(|err| AppError::Internal(format!("Image encode failed: {err}")))?;
    Ok(output.into_inner())
}
