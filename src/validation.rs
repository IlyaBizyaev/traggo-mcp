use chrono::{DateTime, FixedOffset};

use crate::error::AppError;

pub const DEFAULT_PAGE_SIZE: i64 = 50;
pub const MAX_PAGE_SIZE: i64 = 200;

pub fn validate_required_string(name: &str, value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::Validation(format!("{name} must not be empty")));
    }
    Ok(())
}

pub fn validate_optional_required_string(
    name: &str,
    value: Option<&String>,
) -> Result<(), AppError> {
    if let Some(value) = value {
        validate_required_string(name, value)?;
    }
    Ok(())
}

pub fn validate_rfc3339(name: &str, value: &str) -> Result<DateTime<FixedOffset>, AppError> {
    DateTime::parse_from_rfc3339(value).map_err(|_| {
        AppError::Validation(format!(
            "{name} must be an RFC3339 timestamp, got {value:?}"
        ))
    })
}

pub fn validate_optional_rfc3339(name: &str, value: Option<&String>) -> Result<(), AppError> {
    if let Some(value) = value {
        validate_rfc3339(name, value)?;
    }
    Ok(())
}

pub fn validate_time_order(
    start_name: &str,
    start: &str,
    end_name: &str,
    end: &str,
) -> Result<(), AppError> {
    let start = validate_rfc3339(start_name, start)?;
    let end = validate_rfc3339(end_name, end)?;
    if start >= end {
        return Err(AppError::Validation(format!(
            "{start_name} must be before {end_name}"
        )));
    }
    Ok(())
}

pub fn validate_page_size(page_size: Option<i64>) -> Result<i64, AppError> {
    let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    if !(1..=MAX_PAGE_SIZE).contains(&page_size) {
        return Err(AppError::Validation(format!(
            "page_size must be between 1 and {MAX_PAGE_SIZE}"
        )));
    }
    Ok(page_size)
}

pub fn validate_non_negative(name: &str, value: Option<i64>) -> Result<(), AppError> {
    if let Some(value) = value
        && value < 0
    {
        return Err(AppError::Validation(format!("{name} must be non-negative")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_rfc3339() {
        validate_rfc3339("start", "2026-01-01T12:00:00Z").expect("valid timestamp");
    }

    #[test]
    fn rejects_invalid_rfc3339() {
        let err = validate_rfc3339("start", "2026-01-01").expect_err("invalid timestamp");
        assert!(err.to_string().contains("RFC3339"));
    }

    #[test]
    fn rejects_backwards_range() {
        let err = validate_time_order(
            "start",
            "2026-01-02T00:00:00Z",
            "end",
            "2026-01-01T00:00:00Z",
        )
        .expect_err("invalid range");
        assert!(err.to_string().contains("before"));
    }

    #[test]
    fn rejects_large_page_size() {
        let err = validate_page_size(Some(MAX_PAGE_SIZE + 1)).expect_err("too large");
        assert!(err.to_string().contains("page_size"));
    }
}
