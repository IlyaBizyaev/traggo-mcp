use graphql_client::GraphQLQuery;

pub type Time = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "graphql/time_spans.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct TimeSpans;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "graphql/timers.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct Timers;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "graphql/create_time_span.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct CreateTimeSpan;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "graphql/update_time_span.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct UpdateTimeSpan;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "graphql/stop_timer.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct StopTimer;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "graphql/remove_time_span.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct RemoveTimeSpan;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "graphql/tags.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct Tags;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "graphql/create_tag.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct CreateTag;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "graphql/update_tag.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct UpdateTag;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "graphql/remove_tag.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct RemoveTag;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "graphql/suggest_tag_values.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct SuggestTagValues;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "graphql/stats.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct Stats;
