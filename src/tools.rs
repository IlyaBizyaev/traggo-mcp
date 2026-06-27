use rmcp::{
    Json, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    cursor::PageCursor,
    error::AppError,
    graphql,
    traggo_client::TraggoClient,
    validation::{
        validate_optional_required_string, validate_optional_rfc3339, validate_page_size,
        validate_positive_id, validate_required_string, validate_rfc3339, validate_time_order,
        validate_time_order_inclusive,
    },
};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TagInput {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RangeInput {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListTimeSpansInput {
    pub from_inclusive: Option<String>,
    pub to_inclusive: Option<String>,
    /// Maximum number of time spans to return (1-200, defaults to 50).
    pub page_size: Option<i64>,
    /// Opaque pagination token from a previous response's `next_cursor`.
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateTimeSpanInput {
    pub start: String,
    pub end: Option<String>,
    pub tags: Option<Vec<TagInput>>,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateTimeSpanInput {
    pub id: i64,
    pub start: String,
    pub end: Option<String>,
    /// Replaces the time span's tags entirely; omit only to clear all tags.
    pub tags: Option<Vec<TagInput>>,
    pub old_start: Option<String>,
    /// Replaces the time span's note entirely; pass the existing note to keep it.
    pub note: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StopTimerInput {
    pub id: i64,
    pub end: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RemoveTimeSpanInput {
    pub id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateTagInput {
    pub key: String,
    pub color: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateTagInput {
    pub key: String,
    pub new_key: Option<String>,
    pub color: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RemoveTagInput {
    pub key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SuggestTagValuesInput {
    pub key: String,
    pub query: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StatsInput {
    pub ranges: Vec<RangeInput>,
    pub tags: Vec<String>,
    pub exclude_tags: Option<Vec<TagInput>>,
    pub require_tags: Option<Vec<TagInput>>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListTimeSpansOutput {
    pub time_spans: Vec<TimeSpanOutput>,
    /// Opaque token for the next page, present only when more results exist.
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListTimersOutput {
    pub timers: Vec<TimeSpanOutput>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TimeSpanMutationOutput {
    pub time_span: Option<TimeSpanOutput>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TimeSpanOutput {
    pub id: i64,
    pub start: String,
    pub end: Option<String>,
    pub old_start: Option<String>,
    pub note: String,
    pub tags: Vec<TagOutput>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TagsOutput {
    pub tags: Vec<TagDefinitionOutput>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TagMutationOutput {
    pub tag: Option<TagDefinitionOutput>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TagDefinitionOutput {
    pub key: String,
    pub color: String,
    pub usages: i64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SuggestTagValuesOutput {
    pub values: Vec<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct StatsOutput {
    pub ranges: Vec<StatsRangeOutput>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct StatsRangeOutput {
    pub start: String,
    pub end: String,
    pub entries: Vec<StatsEntryOutput>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct StatsEntryOutput {
    pub key: String,
    pub value: String,
    pub time_spend_in_seconds: f64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TagOutput {
    pub key: String,
    pub value: String,
}

#[derive(Clone)]
pub struct TraggoTools {
    client: TraggoClient,
    tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router, instructions = "Purpose-built Traggo time tracking tools backed by Traggo GraphQL.")]
impl ServerHandler for TraggoTools {}

#[tool_router(router = tool_router)]
impl TraggoTools {
    pub fn new(client: TraggoClient) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "list_time_spans",
        description = "List Traggo time spans with optional RFC3339 range filters and bounded pagination. Pass `next_cursor` from a prior response back as `cursor` to page forward."
    )]
    pub async fn list_time_spans(
        &self,
        Parameters(input): Parameters<ListTimeSpansInput>,
    ) -> Result<Json<ListTimeSpansOutput>, String> {
        validate_optional_rfc3339("from_inclusive", input.from_inclusive.as_ref())
            .map_err(tool_error)?;
        validate_optional_rfc3339("to_inclusive", input.to_inclusive.as_ref())
            .map_err(tool_error)?;
        if let (Some(from), Some(to)) = (&input.from_inclusive, &input.to_inclusive) {
            validate_time_order_inclusive("from_inclusive", from, "to_inclusive", to)
                .map_err(tool_error)?;
        }

        let previous = input
            .cursor
            .as_deref()
            .map(PageCursor::decode)
            .transpose()
            .map_err(tool_error)?;
        let page_size =
            validate_page_size(input.page_size.or(previous.as_ref().map(|c| c.page_size)))
                .map_err(tool_error)?;

        let variables = graphql::time_spans::Variables {
            from_inclusive: input.from_inclusive,
            to_inclusive: input.to_inclusive,
            cursor: Some(graphql::time_spans::InputCursor {
                offset: previous.as_ref().map(|c| c.offset),
                start_id: previous.as_ref().map(|c| c.start_id),
                page_size: Some(page_size),
            }),
        };
        self.client
            .list_time_spans(variables)
            .await
            .map_err(tool_error)
            .and_then(list_time_spans_output)
    }

    #[tool(
        name = "list_timers",
        description = "List active Traggo timers/open time spans."
    )]
    pub async fn list_timers(&self) -> Result<Json<ListTimersOutput>, String> {
        self.client
            .list_timers()
            .await
            .map(list_timers_output)
            .map_err(tool_error)
    }

    #[tool(
        name = "create_time_span",
        description = "Create a Traggo time span. Omitting end starts an active timer."
    )]
    pub async fn create_time_span(
        &self,
        Parameters(input): Parameters<CreateTimeSpanInput>,
    ) -> Result<Json<TimeSpanMutationOutput>, String> {
        validate_rfc3339("start", &input.start).map_err(tool_error)?;
        validate_optional_rfc3339("end", input.end.as_ref()).map_err(tool_error)?;
        if let Some(end) = &input.end {
            validate_time_order("start", &input.start, "end", end).map_err(tool_error)?;
        }
        validate_tags(input.tags.as_deref()).map_err(tool_error)?;

        let variables = graphql::create_time_span::Variables {
            start: input.start,
            end: input.end,
            tags: input.tags.map(create_time_span_tags),
            note: input.note,
        };
        self.client
            .create_time_span(variables)
            .await
            .map(|data| {
                Json(TimeSpanMutationOutput {
                    time_span: data.create_time_span.map(create_time_span_output),
                })
            })
            .map_err(tool_error)
    }

    #[tool(
        name = "update_time_span",
        description = "Update an existing Traggo time span by id. This replaces the whole record: the provided note and tags overwrite the existing ones, so resend current values you want to keep."
    )]
    pub async fn update_time_span(
        &self,
        Parameters(input): Parameters<UpdateTimeSpanInput>,
    ) -> Result<Json<TimeSpanMutationOutput>, String> {
        validate_positive_id("id", input.id).map_err(tool_error)?;
        validate_rfc3339("start", &input.start).map_err(tool_error)?;
        validate_optional_rfc3339("end", input.end.as_ref()).map_err(tool_error)?;
        validate_optional_rfc3339("old_start", input.old_start.as_ref()).map_err(tool_error)?;
        if let Some(end) = &input.end {
            validate_time_order("start", &input.start, "end", end).map_err(tool_error)?;
        }
        validate_tags(input.tags.as_deref()).map_err(tool_error)?;

        let variables = graphql::update_time_span::Variables {
            id: input.id,
            start: input.start,
            end: input.end,
            tags: input.tags.map(update_time_span_tags),
            old_start: input.old_start,
            note: input.note,
        };
        self.client
            .update_time_span(variables)
            .await
            .map(|data| {
                Json(TimeSpanMutationOutput {
                    time_span: data.update_time_span.map(update_time_span_output),
                })
            })
            .map_err(tool_error)
    }

    #[tool(
        name = "stop_timer",
        description = "Stop an active Traggo timer by id using an RFC3339 end timestamp."
    )]
    pub async fn stop_timer(
        &self,
        Parameters(input): Parameters<StopTimerInput>,
    ) -> Result<Json<TimeSpanMutationOutput>, String> {
        validate_positive_id("id", input.id).map_err(tool_error)?;
        validate_rfc3339("end", &input.end).map_err(tool_error)?;
        let variables = graphql::stop_timer::Variables {
            id: input.id,
            end: input.end,
        };
        self.client
            .stop_timer(variables)
            .await
            .map(|data| {
                Json(TimeSpanMutationOutput {
                    time_span: data.stop_time_span.map(stop_timer_output),
                })
            })
            .map_err(tool_error)
    }

    #[tool(
        name = "remove_time_span",
        description = "Remove a Traggo time span by id."
    )]
    pub async fn remove_time_span(
        &self,
        Parameters(input): Parameters<RemoveTimeSpanInput>,
    ) -> Result<Json<TimeSpanMutationOutput>, String> {
        validate_positive_id("id", input.id).map_err(tool_error)?;
        let variables = graphql::remove_time_span::Variables { id: input.id };
        self.client
            .remove_time_span(variables)
            .await
            .map(|data| {
                Json(TimeSpanMutationOutput {
                    time_span: data.remove_time_span.map(remove_time_span_output),
                })
            })
            .map_err(tool_error)
    }

    #[tool(
        name = "list_tags",
        description = "List Traggo tag definitions including key, color, and usage count."
    )]
    pub async fn list_tags(&self) -> Result<Json<TagsOutput>, String> {
        self.client
            .list_tags()
            .await
            .map(|data| {
                Json(TagsOutput {
                    tags: data
                        .tags
                        .unwrap_or_default()
                        .into_iter()
                        .map(tags_tag_output)
                        .collect(),
                })
            })
            .map_err(tool_error)
    }

    #[tool(name = "create_tag", description = "Create a Traggo tag definition.")]
    pub async fn create_tag(
        &self,
        Parameters(input): Parameters<CreateTagInput>,
    ) -> Result<Json<TagMutationOutput>, String> {
        validate_required_string("key", &input.key).map_err(tool_error)?;
        validate_required_string("color", &input.color).map_err(tool_error)?;
        let variables = graphql::create_tag::Variables {
            key: input.key,
            color: input.color,
        };
        self.client
            .create_tag(variables)
            .await
            .map(|data| {
                Json(TagMutationOutput {
                    tag: data.create_tag.map(create_tag_output),
                })
            })
            .map_err(tool_error)
    }

    #[tool(
        name = "update_tag",
        description = "Update a Traggo tag key and/or color."
    )]
    pub async fn update_tag(
        &self,
        Parameters(input): Parameters<UpdateTagInput>,
    ) -> Result<Json<TagMutationOutput>, String> {
        validate_required_string("key", &input.key).map_err(tool_error)?;
        validate_optional_required_string("new_key", input.new_key.as_ref()).map_err(tool_error)?;
        validate_required_string("color", &input.color).map_err(tool_error)?;
        let variables = graphql::update_tag::Variables {
            key: input.key,
            new_key: input.new_key,
            color: input.color,
        };
        self.client
            .update_tag(variables)
            .await
            .map(|data| {
                Json(TagMutationOutput {
                    tag: data.update_tag.map(update_tag_output),
                })
            })
            .map_err(tool_error)
    }

    #[tool(
        name = "remove_tag",
        description = "Remove a Traggo tag definition by key."
    )]
    pub async fn remove_tag(
        &self,
        Parameters(input): Parameters<RemoveTagInput>,
    ) -> Result<Json<TagMutationOutput>, String> {
        validate_required_string("key", &input.key).map_err(tool_error)?;
        let variables = graphql::remove_tag::Variables { key: input.key };
        self.client
            .remove_tag(variables)
            .await
            .map(|data| {
                Json(TagMutationOutput {
                    tag: data.remove_tag.map(remove_tag_output),
                })
            })
            .map_err(tool_error)
    }

    #[tool(
        name = "suggest_tag_values",
        description = "Suggest up to 10 historical values for a Traggo tag key and query."
    )]
    pub async fn suggest_tag_values(
        &self,
        Parameters(input): Parameters<SuggestTagValuesInput>,
    ) -> Result<Json<SuggestTagValuesOutput>, String> {
        validate_required_string("key", &input.key).map_err(tool_error)?;
        let variables = graphql::suggest_tag_values::Variables {
            key: input.key,
            query: input.query,
        };
        self.client
            .suggest_tag_values(variables)
            .await
            .map(|data| {
                Json(SuggestTagValuesOutput {
                    values: data.suggest_tag_value.unwrap_or_default(),
                })
            })
            .map_err(tool_error)
    }

    #[tool(
        name = "get_stats",
        description = "Get Traggo tracked seconds grouped by tag keys for explicit RFC3339 ranges."
    )]
    pub async fn get_stats(
        &self,
        Parameters(input): Parameters<StatsInput>,
    ) -> Result<Json<StatsOutput>, String> {
        if input.ranges.is_empty() {
            return Err("validation error: ranges must not be empty".into());
        }
        if input.tags.is_empty() {
            return Err("validation error: tags must not be empty".into());
        }
        for (index, range) in input.ranges.iter().enumerate() {
            validate_time_order(
                &format!("ranges[{index}].start"),
                &range.start,
                &format!("ranges[{index}].end"),
                &range.end,
            )
            .map_err(tool_error)?;
        }
        for (index, tag) in input.tags.iter().enumerate() {
            validate_required_string(&format!("tags[{index}]"), tag).map_err(tool_error)?;
        }
        validate_tags(input.exclude_tags.as_deref()).map_err(tool_error)?;
        validate_tags(input.require_tags.as_deref()).map_err(tool_error)?;

        let variables = graphql::stats::Variables {
            ranges: Some(
                input
                    .ranges
                    .into_iter()
                    .map(|range| graphql::stats::Range {
                        start: range.start,
                        end: range.end,
                    })
                    .collect(),
            ),
            tags: Some(input.tags),
            exclude_tags: input.exclude_tags.map(stats_tags),
            require_tags: input.require_tags.map(stats_tags),
        };
        self.client
            .stats(variables)
            .await
            .map(stats_output)
            .map_err(tool_error)
    }
}

fn list_time_spans_output(
    data: graphql::time_spans::ResponseData,
) -> Result<Json<ListTimeSpansOutput>, String> {
    let cursor = data.time_spans.cursor;
    let next_cursor = if cursor.has_more {
        Some(
            PageCursor {
                offset: cursor.offset,
                start_id: cursor.start_id,
                page_size: cursor.page_size,
            }
            .encode()
            .map_err(tool_error)?,
        )
    } else {
        None
    };
    Ok(Json(ListTimeSpansOutput {
        time_spans: data
            .time_spans
            .time_spans
            .into_iter()
            .map(time_spans_time_span_output)
            .collect(),
        next_cursor,
    }))
}

fn list_timers_output(data: graphql::timers::ResponseData) -> Json<ListTimersOutput> {
    Json(ListTimersOutput {
        timers: data
            .timers
            .unwrap_or_default()
            .into_iter()
            .map(timers_time_span_output)
            .collect(),
    })
}

/// Generates a `TimeSpanOutput` mapper and its tag mapper for one generated query type.
macro_rules! time_span_mapper {
    ($span_fn:ident, $span:path, $tag_fn:ident, $tag:path) => {
        fn $tag_fn(tag: $tag) -> TagOutput {
            TagOutput {
                key: tag.key,
                value: tag.value,
            }
        }

        fn $span_fn(time_span: $span) -> TimeSpanOutput {
            TimeSpanOutput {
                id: time_span.id,
                start: time_span.start,
                end: time_span.end,
                old_start: time_span.old_start,
                note: time_span.note,
                tags: time_span
                    .tags
                    .unwrap_or_default()
                    .into_iter()
                    .map($tag_fn)
                    .collect(),
            }
        }
    };
}

time_span_mapper!(
    time_spans_time_span_output,
    graphql::time_spans::TimeSpansTimeSpansTimeSpans,
    time_spans_tag_output,
    graphql::time_spans::TimeSpansTimeSpansTimeSpansTags
);
time_span_mapper!(
    timers_time_span_output,
    graphql::timers::TimersTimers,
    timers_tag_output,
    graphql::timers::TimersTimersTags
);
time_span_mapper!(
    create_time_span_output,
    graphql::create_time_span::CreateTimeSpanCreateTimeSpan,
    create_time_span_tag_output,
    graphql::create_time_span::CreateTimeSpanCreateTimeSpanTags
);
time_span_mapper!(
    update_time_span_output,
    graphql::update_time_span::UpdateTimeSpanUpdateTimeSpan,
    update_time_span_tag_output,
    graphql::update_time_span::UpdateTimeSpanUpdateTimeSpanTags
);
time_span_mapper!(
    stop_timer_output,
    graphql::stop_timer::StopTimerStopTimeSpan,
    stop_timer_tag_output,
    graphql::stop_timer::StopTimerStopTimeSpanTags
);
time_span_mapper!(
    remove_time_span_output,
    graphql::remove_time_span::RemoveTimeSpanRemoveTimeSpan,
    remove_time_span_tag_output,
    graphql::remove_time_span::RemoveTimeSpanRemoveTimeSpanTags
);

/// Generates a `TagDefinitionOutput` mapper for one generated tag-definition type.
macro_rules! tag_definition_mapper {
    ($fn_name:ident, $ty:path) => {
        fn $fn_name(tag: $ty) -> TagDefinitionOutput {
            TagDefinitionOutput {
                key: tag.key,
                color: tag.color,
                usages: tag.usages,
            }
        }
    };
}

tag_definition_mapper!(tags_tag_output, graphql::tags::TagsTags);
tag_definition_mapper!(create_tag_output, graphql::create_tag::CreateTagCreateTag);
tag_definition_mapper!(update_tag_output, graphql::update_tag::UpdateTagUpdateTag);
tag_definition_mapper!(remove_tag_output, graphql::remove_tag::RemoveTagRemoveTag);

fn stats_output(data: graphql::stats::ResponseData) -> Json<StatsOutput> {
    Json(StatsOutput {
        ranges: data
            .stats
            .unwrap_or_default()
            .into_iter()
            .map(stats_range_output)
            .collect(),
    })
}

fn stats_range_output(range: graphql::stats::StatsStats) -> StatsRangeOutput {
    StatsRangeOutput {
        start: range.start,
        end: range.end,
        entries: range
            .entries
            .unwrap_or_default()
            .into_iter()
            .map(stats_entry_output)
            .collect(),
    }
}

fn stats_entry_output(entry: graphql::stats::StatsStatsEntries) -> StatsEntryOutput {
    StatsEntryOutput {
        key: entry.key,
        value: entry.value,
        time_spend_in_seconds: entry.time_spend_in_seconds,
    }
}

fn validate_tags(tags: Option<&[TagInput]>) -> Result<(), AppError> {
    if let Some(tags) = tags {
        for (index, tag) in tags.iter().enumerate() {
            validate_required_string(&format!("tags[{index}].key"), &tag.key)?;
            validate_required_string(&format!("tags[{index}].value"), &tag.value)?;
        }
    }
    Ok(())
}

fn create_time_span_tags(tags: Vec<TagInput>) -> Vec<graphql::create_time_span::InputTimeSpanTag> {
    tags.into_iter()
        .map(|tag| graphql::create_time_span::InputTimeSpanTag {
            key: tag.key,
            value: tag.value,
        })
        .collect()
}

fn update_time_span_tags(tags: Vec<TagInput>) -> Vec<graphql::update_time_span::InputTimeSpanTag> {
    tags.into_iter()
        .map(|tag| graphql::update_time_span::InputTimeSpanTag {
            key: tag.key,
            value: tag.value,
        })
        .collect()
}

fn stats_tags(tags: Vec<TagInput>) -> Vec<graphql::stats::InputTimeSpanTag> {
    tags.into_iter()
        .map(|tag| graphql::stats::InputTimeSpanTag {
            key: tag.key,
            value: tag.value,
        })
        .collect()
}

fn tool_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{CreateTimeSpanInput, TagInput};

    #[test]
    fn create_time_span_schema_disallows_additional_properties() {
        let schema = schemars::schema_for!(CreateTimeSpanInput);
        let value = serde_json::to_value(schema).expect("schema should serialize");

        assert_eq!(
            value.pointer("/additionalProperties"),
            Some(&Value::Bool(false))
        );
    }

    #[test]
    fn nested_tag_schema_disallows_additional_properties() {
        let schema = schemars::schema_for!(TagInput);
        let value = serde_json::to_value(schema).expect("schema should serialize");

        assert_eq!(
            value.pointer("/additionalProperties"),
            Some(&Value::Bool(false))
        );
    }
}
