use axum::{
    body::Body,
    extract::Extension,
    http::Request,
    middleware::Next,
    response::Response,
};

use crate::errors::AppError;
use crate::middleware::auth::AuthUser;

pub async fn require_auth(
    Extension(_user): Extension<AuthUser>,
    request: Request<Body>,
    next: Next,
) -> Response {
    next.run(request).await
}

pub async fn require_role(
    Extension(user): Extension<AuthUser>,
    roles: Vec<String>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    if !roles.contains(&user.role) {
        return Err(AppError::Forbidden(format!(
            "Role '{}' is not authorized for this action",
            user.role
        )));
    }
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_role_list_contains_admin() {
        let roles = vec![
            "ADMIN".to_string(),
            "OWNER".to_string(),
            "MANAGER".to_string(),
        ];
        assert!(roles.contains(&"ADMIN".to_string()));
        assert!(!roles.contains(&"MECHANIC".to_string()));
    }

    #[test]
    fn test_role_case_sensitivity() {
        let roles = vec!["ADMIN".to_string()];
        assert!(roles.contains(&"ADMIN".to_string()));
        assert!(!roles.contains(&"admin".to_string()));
        assert!(!roles.contains(&"Admin".to_string()));
    }

    #[test]
    fn test_empty_roles_list_denies_all() {
        let roles: Vec<String> = vec![];
        let user_role = "ADMIN".to_string();
        assert!(!roles.contains(&user_role));
    }

    #[test]
    fn test_multiple_roles_authorization() {
        let allowed_roles = vec![
            "ADMIN".to_string(),
            "OWNER".to_string(),
            "MANAGER".to_string(),
            "ACCOUNT_MGR".to_string(),
        ];

        assert!(allowed_roles.contains(&"ADMIN".to_string()));
        assert!(allowed_roles.contains(&"OWNER".to_string()));
        assert!(allowed_roles.contains(&"MANAGER".to_string()));
        assert!(allowed_roles.contains(&"ACCOUNT_MGR".to_string()));
        assert!(!allowed_roles.contains(&"MECHANIC".to_string()));
        assert!(!allowed_roles.contains(&"CASHIER".to_string()));
        assert!(!allowed_roles.contains(&"HR_OFFICER".to_string()));
    }
}
