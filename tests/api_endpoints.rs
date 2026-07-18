use oanda_cli::{Config, ErrorKind, OandaClient};
use serde_json::json;
use std::time::Duration;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn client(server: &MockServer) -> OandaClient {
    let config =
        Config::new("test-access-token", "101-001-123-001").with_datetime_format("RFC3339");
    OandaClient::with_base_urls(&config, server.uri(), server.uri()).unwrap()
}

#[tokio::test]
async fn account_summary_uses_the_expected_endpoint_and_headers() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/accounts/101-001-123-001/summary"))
        .and(header("authorization", "Bearer test-access-token"))
        .and(header("accept-datetime-format", "RFC3339"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"account": {"id": "1"}})))
        .expect(1)
        .mount(&server)
        .await;

    let response = client(&server).await.account().summary().await.unwrap();
    assert_eq!(response["account"]["id"], "1");
}

#[tokio::test]
async fn order_create_posts_the_exact_json_body() {
    let server = MockServer::start().await;
    let body = json!({"order": {
        "type": "MARKET",
        "instrument": "EUR_USD",
        "units": "100",
        "timeInForce": "FOK",
        "positionFill": "DEFAULT"
    }});
    Mock::given(method("POST"))
        .and(path("/v3/accounts/101-001-123-001/orders"))
        .and(body_json(body.clone()))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(json!({"orderCreateTransaction": {"id": "42"}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let response = client(&server).await.orders().create(body).await.unwrap();
    assert_eq!(response["orderCreateTransaction"]["id"], "42");
}

#[tokio::test]
async fn pricing_encodes_query_parameters() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/accounts/101-001-123-001/pricing"))
        .and(query_param("instruments", "EUR_USD,USD_JPY"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"prices": []})))
        .expect(1)
        .mount(&server)
        .await;

    let response = client(&server)
        .await
        .pricing()
        .get("EUR_USD,USD_JPY")
        .await
        .unwrap();
    assert_eq!(response["prices"], json!([]));
}

#[tokio::test]
async fn api_rejections_have_structured_kinds_and_exit_codes() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/accounts"))
        .respond_with(
            ResponseTemplate::new(401).set_body_json(json!({"errorMessage": "Unauthorized"})),
        )
        .mount(&server)
        .await;

    let error = client(&server).await.accounts().list().await.unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Authentication);
    assert_eq!(error.exit_code(), 3);
    assert_eq!(error.structured()["error"]["status"], 401);
}

#[tokio::test]
async fn request_timeout_has_a_stable_error_kind() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/accounts"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(100))
                .set_body_json(json!({"accounts": []})),
        )
        .mount(&server)
        .await;

    let mut config = Config::new("test-access-token", "101-001-123-001");
    config.request_timeout = Duration::from_millis(20);
    let client = OandaClient::with_base_urls(&config, server.uri(), server.uri()).unwrap();
    let error = client.accounts().list().await.unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Timeout);
    assert_eq!(error.exit_code(), 4);
}
