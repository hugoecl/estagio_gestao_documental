use actix_session::Session;
use actix_web::HttpResponse;

pub fn validate_session(session: &Session) -> Result<i32, HttpResponse> {
    let user_id = session.get::<i32>("user_id").unwrap_or(None);

    if let Some(last_renewal) = session.get::<i64>("last_renewal").unwrap_or(None) {
        let now = chrono::Utc::now().timestamp();
        if now - last_renewal > 300 {
            session.renew();
            session.insert("last_renewal", now).unwrap();
        }
    } else {
        session
            .insert("last_renewal", chrono::Utc::now().timestamp())
            .unwrap();
    }

    match user_id {
        Some(id) => Ok(id),
        None => Err(HttpResponse::Unauthorized().finish()),
    }
}

pub fn is_admin(session: &Session) -> Result<i32, HttpResponse> {
    let is_admin = session.get::<bool>("is_admin").unwrap();

    match is_admin {
        Some(true) => Ok(session.get::<i32>("user_id").unwrap().unwrap()),
        _ => Err(HttpResponse::Forbidden().finish()),
    }
}

pub async fn user_can_manage_page(
    pool: &sqlx::MySqlPool,
    user_id: i32,
    page_id: u32,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM user_roles ur
        LEFT JOIN roles r ON r.id = ur.role_id
        LEFT JOIN page_permissions pp ON pp.role_id = ur.role_id AND pp.page_id = ?
        WHERE ur.user_id = ? AND (r.is_admin = 1 OR pp.can_manage_fields = 1)
        "#,
        page_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.count > 0)
}

// Function to get user IDs who have view permission for a specific page
pub async fn get_user_ids_with_view_permission(
    pool: &sqlx::MySqlPool,
    page_id: u32,
) -> Result<Vec<u32>, sqlx::Error> {
    let user_ids = sqlx::query_scalar!(
        r#"
        SELECT DISTINCT ur.user_id
        FROM user_roles ur
        LEFT JOIN roles r ON ur.role_id = r.id
        LEFT JOIN page_permissions pp ON pp.role_id = ur.role_id AND pp.page_id = ?
        WHERE r.is_admin = 1 OR pp.can_view = 1
        "#,
        page_id
    )
    .fetch_all(pool)
    .await?;

    // The query directly returns Vec<u32> as user_id is not nullable
    Ok(user_ids)
}

pub async fn user_can_view_page(
    pool: &sqlx::MySqlPool,
    user_id: i32,
    page_id: u32,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM user_roles ur
        LEFT JOIN roles r ON r.id = ur.role_id
        LEFT JOIN page_permissions pp ON pp.role_id = ur.role_id AND pp.page_id = ?
        WHERE ur.user_id = ? AND (r.is_admin = 1 OR pp.can_view = 1)
        "#,
        page_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.count > 0)
}

pub async fn user_can_create_record(
    pool: &sqlx::MySqlPool,
    user_id: i32,
    page_id: u32,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM user_roles ur
        LEFT JOIN roles r ON r.id = ur.role_id
        LEFT JOIN page_permissions pp ON pp.role_id = ur.role_id AND pp.page_id = ?
        WHERE ur.user_id = ? AND (r.is_admin = 1 OR pp.can_create = 1)
        "#,
        page_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.count > 0)
}

pub async fn user_can_edit_record(
    pool: &sqlx::MySqlPool,
    user_id: i32,
    page_id: u32,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM user_roles ur
        LEFT JOIN roles r ON r.id = ur.role_id
        LEFT JOIN page_permissions pp ON pp.role_id = ur.role_id AND pp.page_id = ?
        WHERE ur.user_id = ? AND (r.is_admin = 1 OR pp.can_edit = 1)
        "#,
        page_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.count > 0)
}

pub async fn user_can_delete_record(
    pool: &sqlx::MySqlPool,
    user_id: i32,
    page_id: u32,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM user_roles ur
        LEFT JOIN roles r ON r.id = ur.role_id
        LEFT JOIN page_permissions pp ON pp.role_id = ur.role_id AND pp.page_id = ?
        WHERE ur.user_id = ? AND (r.is_admin = 1 OR pp.can_delete = 1)
        "#,
        page_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.count > 0)
}

pub async fn user_can_add_to_record(
    pool: &sqlx::MySqlPool,
    user_id: i32,
    page_id: u32,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM user_roles ur
        LEFT JOIN roles r ON r.id = ur.role_id
        LEFT JOIN page_permissions pp ON pp.role_id = ur.role_id AND pp.page_id = ?
        WHERE ur.user_id = ? AND (r.is_admin = 1 OR pp.can_add = 1)
        "#,
        page_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.count > 0)
}
