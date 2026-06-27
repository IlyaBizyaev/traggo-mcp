use std::{env, time::Duration};

use reqwest::Url;

use crate::error::AppError;

const DEFAULT_TIMEOUT_SECONDS: u64 = 30;

#[derive(Clone)]
pub struct Config {
    pub traggo_url: Url,
    pub traggo_token: String,
    pub timeout: Duration,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Self::from_env_vars(|key| env::var(key).ok())
    }

    pub fn from_env_vars(get: impl Fn(&str) -> Option<String>) -> Result<Self, AppError> {
        let traggo_url = required_env(&get, "TRAGGO_URL")?;
        let traggo_token = required_env(&get, "TRAGGO_TOKEN")?;
        let timeout = match get("TRAGGO_TIMEOUT_SECONDS") {
            Some(raw) => {
                let seconds = raw.parse::<u64>().map_err(|_| {
                    AppError::Config("TRAGGO_TIMEOUT_SECONDS must be a positive integer".into())
                })?;
                if seconds == 0 {
                    return Err(AppError::Config(
                        "TRAGGO_TIMEOUT_SECONDS must be greater than zero".into(),
                    ));
                }
                Duration::from_secs(seconds)
            }
            None => Duration::from_secs(DEFAULT_TIMEOUT_SECONDS),
        };

        if traggo_token.trim().is_empty() {
            return Err(AppError::Config("TRAGGO_TOKEN must not be empty".into()));
        }

        Ok(Self {
            traggo_url: Url::parse(&traggo_url)
                .map_err(|err| AppError::Config(format!("TRAGGO_URL is invalid: {err}")))?,
            traggo_token,
            timeout,
        })
    }
}

fn required_env(get: &impl Fn(&str) -> Option<String>, key: &str) -> Result<String, AppError> {
    get(key).ok_or_else(|| AppError::Config(format!("{key} is required")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_for(key: &str) -> Option<String> {
        match key {
            "TRAGGO_URL" => Some("https://traggo.example.com/graphql".into()),
            "TRAGGO_TOKEN" => Some("secret-token".into()),
            _ => None,
        }
    }

    #[test]
    fn loads_required_config() {
        let config = Config::from_env_vars(env_for).expect("config loads");
        assert_eq!(
            config.traggo_url.as_str(),
            "https://traggo.example.com/graphql"
        );
        assert_eq!(config.timeout, Duration::from_secs(DEFAULT_TIMEOUT_SECONDS));
    }

    #[test]
    fn rejects_missing_token() {
        let result = Config::from_env_vars(|key| match key {
            "TRAGGO_URL" => Some("https://traggo.example.com/graphql".into()),
            _ => None,
        });
        let Err(err) = result else {
            panic!("token is required");
        };
        assert!(err.to_string().contains("TRAGGO_TOKEN"));
    }

    #[test]
    fn rejects_bad_timeout() {
        let result = Config::from_env_vars(|key| match key {
            "TRAGGO_URL" => Some("https://traggo.example.com/graphql".into()),
            "TRAGGO_TOKEN" => Some("secret-token".into()),
            "TRAGGO_TIMEOUT_SECONDS" => Some("0".into()),
            _ => None,
        });
        let Err(err) = result else {
            panic!("zero timeout is invalid");
        };
        assert!(err.to_string().contains("greater than zero"));
    }
}
