use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};

use crate::jwt_authentication;

pub async fn patients_authorization(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    dotenvy::dotenv().ok();
    let patient_secret =
        std::env::var("JWT_PATIENT_SECRET").expect("JWT_PATIENT_SECRET is invalid");

    if let Some(cookie_header) = req.headers().get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            let access_token = get_cookie_value(cookie_str, "act");

            if let Some(token) = access_token {
                if let Ok(claims) = jwt_authentication::verify_token(patient_secret, token) {
                    if let Ok(patient_id) = claims.sub.parse::<i32>() {
                        req.extensions_mut().insert(patient_id);
                        return Ok(next.run(req).await);
                    }
                }
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

pub async fn doctors_authorization(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    dotenvy::dotenv().ok();
    let doctor_secret = std::env::var("JWT_DOCTOR_SECRET").expect("JWT_DOCTOR_SECRET is invalid");

    if let Some(cookie_header) = req.headers().get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            let access_token = get_cookie_value(cookie_str, "act");

            if let Some(token) = access_token {
                if let Ok(claims) = jwt_authentication::verify_token(doctor_secret, token) {
                    if let Ok(doctor_id) = claims.sub.parse::<i32>() {
                        req.extensions_mut().insert(doctor_id);
                        return Ok(next.run(req).await);
                    }
                }
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

fn get_cookie_value(cookie_header: &str, key: &str) -> Option<String> {
    cookie_header.split("; ").find_map(|cookie| {
        let mut parts = cookie.splitn(2, "=");
        let name = parts.next()?.trim();
        let value = parts.next()?.trim();
        if name == key {
            Some(value.to_string())
        } else {
            None
        }
    })
}
