use std::{
    env,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{DateTime, Utc};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;
use url_shortener::{
    api,
    config::Settings,
    infrastructure::cassandra_repository::CassandraRepository,
    short_link::{
        domain::{DestinationUrl, ShortCode},
        service::{InsertOutcome, ShortLinkRepository, ShortLinkService, StorageError},
    },
};

async fn repository() -> CassandraRepository {
    let settings = Settings::from_env().expect("Cassandra test settings are valid");
    CassandraRepository::connect(&settings)
        .await
        .expect("test Cassandra is reachable and schema initializes")
}

fn app(repository: CassandraRepository) -> Router {
    api::router(ShortLinkService::new(repository))
}

async fn post_link(router: Router, destination_url: &str) -> axum::response::Response {
    router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/links")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "destination_url": destination_url }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn create_deduplicates_canonical_urls_and_returns_201_then_200() {
    let router = app(repository().await);
    let id = uuid::Uuid::new_v4();
    let first_url = format!("HTTPS://Example.COM:443/{id}/../article");

    let first = post_link(router.clone(), &first_url).await;
    assert_eq!(first.status(), StatusCode::CREATED);
    let first_body = read_json(first).await;
    let code = first_body["code"].as_str().unwrap().to_owned();
    assert_eq!(first_body["short_path"], format!("/{code}"));

    let duplicate = post_link(router, "https://example.com/article").await;
    assert_eq!(duplicate.status(), StatusCode::OK);
    assert_eq!(read_json(duplicate).await["code"], code);
}

#[tokio::test]
async fn concurrent_submissions_through_independent_api_repositories_share_one_mapping() {
    let first_repository = repository().await;
    let settings = Settings::from_env().unwrap();
    let second_repository = CassandraRepository::connect(&settings)
        .await
        .expect("second API replica can connect to Cassandra");
    let first_router = app(first_repository);
    let second_router = app(second_repository);
    let destination = format!("https://parallel.example/{}", uuid::Uuid::new_v4());
    let requests = (0..24).map(|index| {
        let router = if index % 2 == 0 {
            first_router.clone()
        } else {
            second_router.clone()
        };
        post_link(router, &destination)
    });
    let responses = futures_util::future::join_all(requests).await;

    let mut codes = Vec::new();
    let mut created_count = 0;
    for response in responses {
        assert!(matches!(
            response.status(),
            StatusCode::CREATED | StatusCode::OK
        ));
        if response.status() == StatusCode::CREATED {
            created_count += 1;
        }
        codes.push(
            read_json(response).await["code"]
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }

    assert_eq!(created_count, 1);
    assert!(codes.iter().all(|code| code == &codes[0]));
    let stored = first_router
        .oneshot(
            Request::builder()
                .uri(format!("/{}", codes[0]))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(stored.status(), StatusCode::MOVED_PERMANENTLY);
    assert_eq!(stored.headers()["location"], destination);
}

#[tokio::test]
async fn an_existing_hash_prefix_collision_extends_the_code() {
    let repository = repository().await;
    let destination = DestinationUrl::parse(&format!(
        "https://collision.example/{}",
        uuid::Uuid::new_v4()
    ))
    .unwrap();
    let occupied = DestinationUrl::parse(&format!(
        "https://occupied.example/{}",
        uuid::Uuid::new_v4()
    ))
    .unwrap();
    let prefix = ShortCode::from_digest_prefix(&destination.digest(), 12).unwrap();
    assert_eq!(
        repository
            .insert_if_absent(&prefix, &occupied, Utc::now())
            .await
            .unwrap(),
        InsertOutcome::Inserted
    );

    let result = ShortLinkService::new(repository)
        .create(&destination)
        .await
        .unwrap();

    assert!(result.was_created);
    assert_eq!(result.code.as_str().len(), 13);
    assert!(result.code.as_str().starts_with(prefix.as_str()));
}

#[tokio::test]
async fn redirect_is_301_with_location_and_cache_policy() {
    let router = app(repository().await);
    let destination = format!("https://redirect.example/{}?q=one", uuid::Uuid::new_v4());
    let created = post_link(router.clone(), &destination).await;
    let code = read_json(created).await["code"]
        .as_str()
        .unwrap()
        .to_owned();

    let redirected = router
        .oneshot(
            Request::builder()
                .uri(format!("/{code}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(redirected.status(), StatusCode::MOVED_PERMANENTLY);
    assert_eq!(redirected.headers()["location"], destination);
    assert_eq!(
        redirected.headers()["cache-control"],
        "public, max-age=3600"
    );
}

#[tokio::test]
async fn unknown_or_malformed_codes_return_not_found() {
    let router = app(repository().await);
    for path in ["/ZZZZZZZZZZZZ", "/not-a-code", "/0123456789A-"] {
        let response = router
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "path {path}");
    }
}

#[tokio::test]
async fn api_service_does_not_serve_the_frontend() {
    let response = app(repository().await)
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[derive(Clone, Default)]
struct UnavailableRepository;

impl ShortLinkRepository for UnavailableRepository {
    async fn insert_if_absent(
        &self,
        _code: &ShortCode,
        _destination: &DestinationUrl,
        _created_at: DateTime<Utc>,
    ) -> Result<InsertOutcome, StorageError> {
        Err(StorageError)
    }

    async fn find_by_code(&self, _code: &ShortCode) -> Result<Option<String>, StorageError> {
        Err(StorageError)
    }

    async fn is_ready(&self) -> Result<(), StorageError> {
        Err(StorageError)
    }
}

#[tokio::test]
async fn storage_outage_returns_503_for_create_resolve_and_readiness() {
    let router = api::router(ShortLinkService::new(UnavailableRepository));

    let create = post_link(router.clone(), "https://unavailable.example/").await;
    assert_eq!(create.status(), StatusCode::SERVICE_UNAVAILABLE);
    let resolve = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/0123456789AB")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolve.status(), StatusCode::SERVICE_UNAVAILABLE);
    let ready = router
        .oneshot(
            Request::builder()
                .uri("/health/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ready.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn mapping_survives_an_actual_api_process_restart() {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let mut first_process = ApiProcess::start(port);
    first_process.wait_until_ready(port);

    let destination = format!("https://restart.example/{}", uuid::Uuid::new_v4());
    let body = json!({ "destination_url": &destination }).to_string();
    let created = request(port, "POST", "/api/v1/links", &body);
    assert!(created.starts_with("HTTP/1.1 201"), "response: {created}");
    let (_, response_body) = created.split_once("\r\n\r\n").unwrap();
    let code = serde_json::from_str::<Value>(response_body).unwrap()["code"]
        .as_str()
        .unwrap()
        .to_owned();

    first_process.stop();
    let mut restarted_process = ApiProcess::start(port);
    restarted_process.wait_until_ready(port);

    let redirected = request(port, "GET", &format!("/{code}"), "");
    assert!(
        redirected.starts_with("HTTP/1.1 301"),
        "response: {redirected}"
    );
    let location = redirected
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("location").then(|| value.trim())
        })
        .unwrap();
    assert_eq!(location, destination);

    restarted_process.stop();
}

#[test]
fn invalid_runtime_configuration_exits_with_a_safe_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_url-shortener"))
        .env("APP_LISTEN_ADDR", "127.0.0.1:0")
        .env("CASSANDRA_CONTACT_POINTS", "localhost:invalid")
        .env("RUST_LOG", "error")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("test-password"));
}

struct ApiProcess {
    child: Child,
}

impl ApiProcess {
    fn start(port: u16) -> Self {
        let child = Command::new(env!("CARGO_BIN_EXE_url-shortener"))
            .env("APP_LISTEN_ADDR", format!("127.0.0.1:{port}"))
            .env("RUST_LOG", "error")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        Self { child }
    }

    fn wait_until_ready(&mut self, port: u16) {
        for _ in 0..200 {
            if let Some(status) = self.child.try_wait().unwrap() {
                panic!("API process exited before becoming ready: {status}");
            }
            if try_request(port, "GET", "/health/ready", "")
                .is_ok_and(|response| response.starts_with("HTTP/1.1 204"))
            {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
        panic!("API process did not become ready in time");
    }

    fn stop(&mut self) {
        #[cfg(unix)]
        {
            let result = unsafe { libc::kill(self.child.id() as libc::pid_t, libc::SIGTERM) };
            assert_eq!(result, 0, "failed to send SIGTERM to API process");
            assert!(
                self.child.wait().unwrap().success(),
                "API process should shut down gracefully on SIGTERM"
            );
        }

        #[cfg(not(unix))]
        {
            self.child.kill().unwrap();
            self.child.wait().unwrap();
        }
    }
}

impl Drop for ApiProcess {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn request(port: u16, method: &str, path: &str, body: &str) -> String {
    try_request(port, method, path, body).expect("API request succeeds")
}

fn try_request(port: u16, method: &str, path: &str, body: &str) -> std::io::Result<String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

#[tokio::test]
async fn invalid_destinations_return_422_without_writing_rows() {
    let router = app(repository().await);
    let too_long = format!("https://oversize.example/{}", "a".repeat(2049));
    for destination in [
        "javascript:alert(1)",
        "file:///etc/passwd",
        "https://user:pass@example.com/",
        "https://example.com/with\ncontrol",
    ] {
        let response = post_link(router.clone(), destination).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
    assert_eq!(
        post_link(router.clone(), &too_long).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let invalid_json = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/links")
                .header("content-type", "application/json")
                .body(Body::from("{"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_json.status(), StatusCode::BAD_REQUEST);
}
