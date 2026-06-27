use graphql_client::Error as GraphqlError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("http error: {0}")]
    Http(String),
    #[error("graphql error: {0}")]
    Graphql(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}

impl AppError {
    pub fn redact(self, token: &str) -> Self {
        match self {
            Self::Config(message) => Self::Config(redact_token(&message, token)),
            Self::Validation(message) => Self::Validation(redact_token(&message, token)),
            Self::Http(message) => Self::Http(redact_token(&message, token)),
            Self::Graphql(message) => Self::Graphql(redact_token(&message, token)),
            Self::Serialization(message) => Self::Serialization(redact_token(&message, token)),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value.to_string())
    }
}

pub fn normalize_graphql_errors(errors: Vec<GraphqlError>) -> AppError {
    let messages = errors
        .into_iter()
        .map(|error| error.message)
        .collect::<Vec<_>>()
        .join("; ");
    AppError::Graphql(messages)
}

pub fn redact_token(message: &str, token: &str) -> String {
    if token.is_empty() {
        return message.to_owned();
    }
    message.replace(token, "[REDACTED]")
}

#[cfg(test)]
mod tests {
    use graphql_client::Error;

    use super::*;

    #[test]
    fn normalizes_graphql_error_messages() {
        let err = normalize_graphql_errors(vec![Error {
            message: "not authorized".into(),
            locations: None,
            path: None,
            extensions: None,
        }]);
        assert_eq!(err.to_string(), "graphql error: not authorized");
    }

    #[test]
    fn redacts_token_from_error() {
        assert_eq!(
            redact_token("Authorization traggo abc123 failed", "abc123"),
            "Authorization traggo [REDACTED] failed"
        );
    }
}
