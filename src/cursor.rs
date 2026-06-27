use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Opaque pagination token round-tripped between `list_time_spans` calls.
///
/// The encoded form intentionally hides Traggo's raw cursor internals
/// (`offset`/`start_id`/`page_size`) from callers, who should treat it as
/// an opaque string and pass it back verbatim to fetch the next page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageCursor {
    pub offset: i64,
    pub start_id: i64,
    pub page_size: i64,
}

impl PageCursor {
    pub fn encode(&self) -> Result<String, AppError> {
        let json = serde_json::to_vec(self)?;
        Ok(URL_SAFE_NO_PAD.encode(json))
    }

    pub fn decode(token: &str) -> Result<Self, AppError> {
        let bytes = URL_SAFE_NO_PAD
            .decode(token)
            .map_err(|_| AppError::Validation("cursor is not a valid pagination token".into()))?;
        serde_json::from_slice(&bytes)
            .map_err(|_| AppError::Validation("cursor is not a valid pagination token".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_cursor() {
        let cursor = PageCursor {
            offset: 50,
            start_id: 1234,
            page_size: 50,
        };
        let token = cursor.encode().expect("encode");
        let decoded = PageCursor::decode(&token).expect("decode");
        assert_eq!(decoded.offset, 50);
        assert_eq!(decoded.start_id, 1234);
        assert_eq!(decoded.page_size, 50);
    }

    #[test]
    fn rejects_garbage_token() {
        let err = PageCursor::decode("not a token!!!").expect_err("invalid token");
        assert!(err.to_string().contains("pagination token"));
    }
}
