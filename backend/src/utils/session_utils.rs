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

pub fn admin_only(session: &Session) -> Result<(), HttpResponse> {
    let is_admin = session.get::<bool>("is_admin").unwrap();

    match is_admin {
        Some(true) => Ok(()),
        _ => Err(HttpResponse::Forbidden().finish()),
    }
}
