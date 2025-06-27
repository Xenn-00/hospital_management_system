#[macro_export]
macro_rules! parse_visit_type {
    ($visit_type:expr) => {
        match $visit_type.to_lowercase().as_str() {
            "bpjs" => Ok(VisitType::BPJS),
            "common" => Ok(VisitType::COMMON),
            _ => Err(AppError::BadRequest(format!("Unknown visit type: {}", $visit_type)).into()),
        }
    };
}

#[macro_export]
macro_rules! format_option_dt {
    ($dt:expr) => {{
        let naive_dt =
            NaiveDateTime::parse_from_str(&$dt.unwrap().to_string(), "%Y-%m-%d %H:%M:%S%.f")
                .unwrap();

        let utc = DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc);
        let local = utc.with_timezone(&Local);
        local.format("%d-%m-%Y %H:%M:%S").to_string()
    }};
}
#[macro_export]
macro_rules! format_created_at {
    ($created_at:expr) => {{
        let naive_dt =
            NaiveDateTime::parse_from_str(&$created_at.to_string(), "%Y-%m-%d %H:%M:%S%.f")
                .unwrap();

        let utc = DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc);
        let local = utc.with_timezone(&Local);
        local.format("%d-%m-%Y %H:%M:%S").to_string()
    }};
}

#[macro_export]
macro_rules! json_ok {
    ($msg:expr, $data:expr, $req_id:expr) => {
        Ok(axum::Json($crate::infra::api::ApiResponse {
            message: $msg.to_string(),
            data: Some($data),
            request_id: $req_id.0.clone(),
            errors: None,
        }))
    };
}

#[macro_export]
macro_rules! json_error {
    ($status:expr, $msg:expr, $req_id:expr) => {{
        use axum::Json;
        use axum::response::{IntoResponse, Response};
        use $crate::dto::response::ApiResponse;

        let body = ApiResponse::<()> {
            message: $msg.to_string(),
            data: None,
            request_id: $req_id.0.clone(),
            errors: None,
        };

        ($status, Json(body)).into_response()
    }};
}
