use graphql_client::{GraphQLQuery, Response};
use reqwest::{StatusCode, header};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    config::Config,
    error::{AppError, normalize_graphql_errors},
    graphql,
};

#[derive(Clone)]
pub struct TraggoClient {
    http: reqwest::Client,
    config: Config,
}

impl TraggoClient {
    pub fn new(config: Config) -> Result<Self, AppError> {
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|err| AppError::Http(err.to_string()).redact(&config.traggo_token))?;
        Ok(Self { http, config })
    }

    async fn request<Q>(&self, variables: Q::Variables) -> Result<Q::ResponseData, AppError>
    where
        Q: GraphQLQuery,
        Q::Variables: Serialize,
        Q::ResponseData: DeserializeOwned,
    {
        // `type_name` yields the query struct path (e.g. `..::graphql::TimeSpans`);
        // it never contains request data or the auth token.
        let operation = std::any::type_name::<Q>();
        tracing::debug!(operation, url = %self.config.traggo_url, "sending Traggo request");

        let request_body = Q::build_query(variables);
        let authorization = format!("traggo {}", self.config.traggo_token);
        let response = self
            .http
            .post(self.config.traggo_url.clone())
            .header(header::AUTHORIZATION, authorization)
            .json(&request_body)
            .send()
            .await
            .map_err(|err| AppError::Http(err.to_string()).redact(&self.config.traggo_token))?;

        let status = response.status();
        tracing::debug!(operation, %status, "received Traggo response");
        let body = response
            .text()
            .await
            .map_err(|err| AppError::Http(err.to_string()).redact(&self.config.traggo_token))?;

        normalize_http_status(status, &body)
            .map_err(|err| err.redact(&self.config.traggo_token))?;

        let response: Response<Q::ResponseData> = serde_json::from_str(&body).map_err(|err| {
            AppError::Serialization(err.to_string()).redact(&self.config.traggo_token)
        })?;
        normalize_graphql_response(response).map_err(|err| err.redact(&self.config.traggo_token))
    }

    pub async fn list_time_spans(
        &self,
        variables: graphql::time_spans::Variables,
    ) -> Result<graphql::time_spans::ResponseData, AppError> {
        self.request::<graphql::TimeSpans>(variables).await
    }

    pub async fn list_timers(&self) -> Result<graphql::timers::ResponseData, AppError> {
        self.request::<graphql::Timers>(graphql::timers::Variables)
            .await
    }

    pub async fn create_time_span(
        &self,
        variables: graphql::create_time_span::Variables,
    ) -> Result<graphql::create_time_span::ResponseData, AppError> {
        self.request::<graphql::CreateTimeSpan>(variables).await
    }

    pub async fn update_time_span(
        &self,
        variables: graphql::update_time_span::Variables,
    ) -> Result<graphql::update_time_span::ResponseData, AppError> {
        self.request::<graphql::UpdateTimeSpan>(variables).await
    }

    pub async fn stop_timer(
        &self,
        variables: graphql::stop_timer::Variables,
    ) -> Result<graphql::stop_timer::ResponseData, AppError> {
        self.request::<graphql::StopTimer>(variables).await
    }

    pub async fn remove_time_span(
        &self,
        variables: graphql::remove_time_span::Variables,
    ) -> Result<graphql::remove_time_span::ResponseData, AppError> {
        self.request::<graphql::RemoveTimeSpan>(variables).await
    }

    pub async fn list_tags(&self) -> Result<graphql::tags::ResponseData, AppError> {
        self.request::<graphql::Tags>(graphql::tags::Variables)
            .await
    }

    pub async fn create_tag(
        &self,
        variables: graphql::create_tag::Variables,
    ) -> Result<graphql::create_tag::ResponseData, AppError> {
        self.request::<graphql::CreateTag>(variables).await
    }

    pub async fn update_tag(
        &self,
        variables: graphql::update_tag::Variables,
    ) -> Result<graphql::update_tag::ResponseData, AppError> {
        self.request::<graphql::UpdateTag>(variables).await
    }

    pub async fn remove_tag(
        &self,
        variables: graphql::remove_tag::Variables,
    ) -> Result<graphql::remove_tag::ResponseData, AppError> {
        self.request::<graphql::RemoveTag>(variables).await
    }

    pub async fn suggest_tag_values(
        &self,
        variables: graphql::suggest_tag_values::Variables,
    ) -> Result<graphql::suggest_tag_values::ResponseData, AppError> {
        self.request::<graphql::SuggestTagValues>(variables).await
    }

    pub async fn stats(
        &self,
        variables: graphql::stats::Variables,
    ) -> Result<graphql::stats::ResponseData, AppError> {
        self.request::<graphql::Stats>(variables).await
    }
}

fn normalize_http_status(status: StatusCode, body: &str) -> Result<(), AppError> {
    if status.is_success() {
        return Ok(());
    }
    Err(AppError::Http(format!(
        "Traggo returned HTTP {status}: {}",
        trim_body(body)
    )))
}

pub fn normalize_graphql_response<T>(response: Response<T>) -> Result<T, AppError> {
    if let Some(errors) = response.errors
        && !errors.is_empty()
    {
        return Err(normalize_graphql_errors(errors));
    }
    response
        .data
        .ok_or_else(|| AppError::Graphql("missing data in GraphQL response".into()))
}

fn trim_body(body: &str) -> String {
    const MAX_BODY_CHARS: usize = 500;
    let trimmed = body.trim();
    if trimmed.chars().count() <= MAX_BODY_CHARS {
        return trimmed.to_owned();
    }
    let mut truncated = trimmed.chars().take(MAX_BODY_CHARS).collect::<String>();
    truncated.push_str("...");
    truncated
}

#[cfg(test)]
mod tests {
    use graphql_client::Error;
    use serde_json::{Value, json};

    use super::*;

    #[test]
    fn returns_data_when_response_is_successful() {
        let data = json!({"ok": true});
        let response = Response {
            data: Some(data.clone()),
            errors: None,
            extensions: None,
        };
        assert_eq!(normalize_graphql_response(response).expect("data"), data);
    }

    #[test]
    fn returns_graphql_errors_before_data() {
        let response: Response<Value> = Response {
            data: Some(json!({"ok": true})),
            errors: Some(vec![Error {
                message: "token bad".into(),
                locations: None,
                path: None,
                extensions: None,
            }]),
            extensions: None,
        };
        let err = normalize_graphql_response(response).expect_err("error wins");
        assert!(err.to_string().contains("token bad"));
    }

    #[test]
    fn rejects_missing_data() {
        let response: Response<Value> = Response {
            data: None,
            errors: None,
            extensions: None,
        };
        let err = normalize_graphql_response(response).expect_err("missing data");
        assert!(err.to_string().contains("missing data"));
    }
}
