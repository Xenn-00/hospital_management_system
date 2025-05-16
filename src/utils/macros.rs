#[macro_export]
macro_rules! parse_visit_type {
    ($visit_type:expr) => {
        match $visit_type.to_lowercase().as_str() {
            "bpjs" => Ok(VisitType::BPJS),
            "common" => Ok(VisitType::COMMON),
            other => Err(AppError::BadRequest(format!("Unknown visit type: {}", other)).into()),
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
