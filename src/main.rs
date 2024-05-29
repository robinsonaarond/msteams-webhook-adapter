mod adaptive_card;
mod buildkite;
mod snitch;

use std::{collections::HashMap, env};

use axum::{
    body::Body,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_secretsmanager::Client;

use buildkite::BuildData;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use snitch::DmsData;

use crate::adaptive_card::AdaptiveCardData;

const BASE_URL: &'static str = "https://<yoursite>.webhook.office.com/webhookb2/";

#[derive(Clone)]
struct TeamsChannelUrl {
    name: String,
    url: String,
}

#[derive(Clone)]
struct AppState {
    whitelist: Vec<String>,
    channels: Vec<TeamsChannelUrl>,
    base_url: String,
    api_key: String,
}

#[derive(Deserialize)]
struct QueryParams {
    #[serde(rename = "apiKey")]
    api_key: Option<String>,
    channel: Option<String>,
}

#[derive(Serialize)]
struct HealthCheckResponse {
    status: String,
    url_count: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct PostData {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum WebhookData {
    TypeA(BuildData),
    TypeB(PostData),
}

pub async fn make_post_request<T>(url: String, data: T) -> Result<(), reqwest::Error>
where
    T: Serialize + std::fmt::Debug,
{
    let client = reqwest::Client::new();

    println!("Ready to send post request to {:?}: {:?}", url, data);
    // Send the POST request
    let res = client.post(url).json(&data).send().await?;

    // Check the response status and optionally handle the response body
    if res.status().is_success() {
        println!("Request successful.");
    } else {
        println!("Request failed with status: {}", res.status());
    }

    Ok(())
}

async fn process_standard_webhook(url: String, data_object: PostData) {
    let _res = make_post_request(url, data_object).await;
}

async fn process_adaptivecard_webhook(url: String, data_object: BuildData) {
    let adaptive_card_data: AdaptiveCardData = data_object.into();
    let _res = make_post_request(url, adaptive_card_data).await;
}

fn get_webhook_url(channels: Vec<TeamsChannelUrl>, channel: String) -> String {
    channels
        .iter()
        .find(|c| c.name == channel)
        .map_or(format!("No matching URL found for {channel}."), |c| {
            c.url.clone()
        })
}

fn build_dms_post_data(data_object: DmsData) -> PostData {
    println!("Got DMS webhook");
    let snitch_id = data_object.data.snitch.token;
    let snitch_name = data_object.data.snitch.name;
    let href =
        format!("<a href=\"https://deadmanssnitch.com/snitches/{snitch_id},\">{snitch_name}</a> ");
    let data_string = match data_object.snitch_type {
        snitch::SnitchType::Missing => format!("? {href} is missing"),
        snitch::SnitchType::Reporting => format!("✓ {href} is reporting."),
        snitch::SnitchType::Paused => format!("P {href} is paused."),
    };
    PostData { text: data_string }
}

fn teams_urls_to_array(channels: &[TeamsChannelUrl]) -> Vec<String> {
    channels
        .into_iter()
        .map(|channel| {
            let parts: Vec<&str> = channel.url.split(BASE_URL).collect();
            parts[1].to_string()
        })
        .collect()
}

async fn handle_webhook(
    Query(params): Query<QueryParams>,
    State(state): State<AppState>,
    Json(data): Json<WebhookData>,
) -> impl IntoResponse {
    let key = params.api_key.unwrap_or("none".to_string());
    let chan = params.channel.unwrap_or("none".to_string());
    let url = get_webhook_url(state.channels, chan);
    if key == state.api_key {
        println!("API Key matches. Processing...");
        match data {
            WebhookData::TypeA(build_data) => process_adaptivecard_webhook(url, build_data).await,
            //_ => return Err(StatusCode::BAD_REQUEST),
            _ => {
                return Err(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from("Invalid unsupported data type."))
                    .unwrap())
            }
        }
        Ok(Response::builder()
            .status(200)
            .body(Body::from("Webhook processed\n"))
            .unwrap())
    } else {
        Err(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("API Key Mismatch\n"))
            .unwrap())
    }
}

async fn handle_webhookb2(
    State(state): State<AppState>,
    Path(webhook_path): Path<String>,
    Json(data): Json<WebhookData>,
) -> impl IntoResponse {
    println!("Processing webhookb2 path");
    let whitelist = state.whitelist;
    println!("Got a web call");
    if whitelist.contains(&webhook_path.to_string()) {
        println!("Path matches whitelist. Processing...");
        // If we have a channel=asfgasg on the query, prefer that
        let url = state.base_url.to_owned() + &webhook_path;

        match data {
            WebhookData::TypeA(build_data) => process_adaptivecard_webhook(url, build_data).await,
            WebhookData::TypeB(post_data) => process_standard_webhook(url, post_data).await,
        }
    } else {
        println!("Path {} is not in whitelist.", webhook_path);
    }
    // TODO: I should return different responses probably, move this into the if blocks
    Response::builder()
        .status(200)
        .body(Body::from("Webhook processed\n"))
        .unwrap()
}

async fn handle_webhook_dms(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
    Path(webhook_path): Path<String>,
    Json(data): Json<DmsData>,
) -> impl IntoResponse {
    let key = params.api_key.unwrap_or("none".to_string());
    if key == state.api_key {
        println!("API Key matches. Processing...");
        let post_data = build_dms_post_data(data);
        let channel = webhook_path.rsplit_once('/').unwrap().1;
        let _res =
            make_post_request(get_webhook_url(state.channels, channel.into()), post_data).await;
        Response::builder()
            .status(200)
            .body(Body::from("Webhook processed\n"))
            .unwrap()
    } else {
        Response::builder()
            .status(403)
            .body(Body::from("API Key mismatch\n"))
            .unwrap()
    }
}

async fn handle_health_check(State(state): State<AppState>) -> impl IntoResponse {
    Json(HealthCheckResponse {
        status: "success".to_string(),
        url_count: state.whitelist.len().to_string(),
    })
}

async fn aws_client() -> Client {
    let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
    let config = aws_config::from_env().region(region_provider).load().await;
    println!("AWS Client connection established");
    Client::new(&config)
}

async fn get_teams_channels() -> Vec<TeamsChannelUrl> {
    let client = aws_client().await;
    let secret_key_name = "ops/production/teams-webhook-adapter/webhooks.json";
    let resp = client
        .get_secret_value()
        .secret_id(secret_key_name)
        .send()
        .await
        .unwrap();
    // GPT copypasta.  TODO: Review
    // Deserialize JSON string into a HashMap
    let map: HashMap<String, String> =
        serde_json::from_str(&resp.secret_string.unwrap()).expect("JSON was not well-formatted");

    // Convert HashMap into Vec<TeamsChannelUrl>
    let channels: Vec<TeamsChannelUrl> = map
        .into_iter()
        .map(|(name, url)| TeamsChannelUrl { name, url })
        .collect();
    println!("Teams channels processed.");
    channels
}

async fn build_app_state(base_url: String) -> AppState {
    let channels = get_teams_channels().await;
    println!("Got channels: {:?}", &channels.len());
    let whitelist = teams_urls_to_array(&channels);
    AppState {
        whitelist: whitelist,
        api_key: env::var("API_KEY").unwrap_or("<defaultapikey>".to_string()),
        base_url,
        channels,
    }
}

fn new_app(app_state: AppState) -> Router {
    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .route("/webhookb2/*webhook_path", post(handle_webhookb2))
        .route("/api/webhook/*webhook_path", post(handle_webhook_dms))
        .route("/check", get(handle_health_check))
        .with_state(app_state);
    app
}

#[tokio::main]
async fn main() {
    // Build list of webhook paths that we're willing to process
    let app_state = build_app_state(BASE_URL.to_string()).await;
    // build our application with a single route
    let app = new_app(app_state);
    //.route("/webhook", post(handle_webhook))
    //.route("/webhookb2/*webhook_path", post(handle_webhookb2))
    //.route("/api/webhook/*webhook_path", post(handle_webhook_dms))
    //.route("/check", get(handle_health_check))
    //.with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    println!("Listening on port 3000...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// TESTS

#[cfg(test)]
mod tests {
    use axum::body;
    use serde::de::Error;
    use serde_json::Value;

    async fn into_response_json_status<T>(res: T) -> Result<String, serde_json::Error>
    where
        T: IntoResponse,
    {
        let response = res.into_response();
        // Aggregate the body data into a single chunk, handling errors
        let body = response.into_body();
        let bytes = body::to_bytes(body, 100).await.unwrap();

        // Parse the bytes as JSON
        let json: Value = serde_json::from_slice(&bytes)?;

        // Extract the "status" field as a string
        let status = json
            .get("status")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| serde_json::Error::custom("Status field not found or not a string"));

        status
    }

    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let app_state = build_app_state("127.0.0.1".to_string()).await;
        let response_string =
            into_response_json_status(handle_health_check(axum::extract::State(app_state)).await)
                .await;
        assert_eq!(response_string.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_snitch() {
        // ensure that snitch object translation is unchanged
        //assert_eq!(build_dms_post_data(data_object: DmsData) -> PostData )
    }

    // // Integration tests
    // #[tokio::test]
    // async fn test_post_data() {
    //     let mut server = mockito::Server::new();
    //     let mock = server
    //         .mock("POST", "/gtfs")
    //         .with_status(200)
    //         .with_body("asdf")
    //         .create();
    //     let _ = make_post_request(server.url(), PostData { text: "asdf".to_string() }).await;
    //     mock.assert();
    //     //assert_eq!(post_data_response, "asdf");
    // }
}

// Ideas for unit tests:
// Known object should return a known error
// Wrong api key

//  Two layers, where you call the handlers specifically like 312
//  Should have at least one where it launches the whole self and tries to respond (more like an integration test)

#[cfg(test)]
mod moar_tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tokio;
    use tower::ServiceExt; // for `app.oneshot()` method
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_handler_with_mocked_api() {
        // Start a Wiremock server
        let mock_server = MockServer::start().await;

        // Create a mock for the external API call
        let mock_response = "mocked response";
        let mock = Mock::given(method("GET"))
            .and(path("/api"))
            .respond_with(ResponseTemplate::new(200).set_body_string(mock_response));
        mock_server.register(mock).await;

        let base_url = mock_server.uri();
        let app_state = build_app_state(base_url).await;
        let app = new_app(app_state);
        //let _ = make_post_request(server.url(), PostData { text: "asdf".to_string() }).await;

        // Create a request
        let webhook_end = "<whateveryouneedhere>";
        let request_body = serde_json::json!({
            "text": "Whatever"
        });
        let request = Request::builder()
            .uri(format!("/webhookb2/{webhook_end}"))
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // Send the request to the app
        let response = app.oneshot(request).await.unwrap();

        // Verify the response
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert_eq!(body, mock_response);
    }
}
