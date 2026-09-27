use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use url_shortener::{
    api, config::Settings, infrastructure::cassandra_repository::CassandraRepository,
    short_link::service::ShortLinkService,
};

#[tokio::main]
async fn main() {
    init_tracing();
    if let Err(message) = run_server().await {
        error!("{message}");
        std::process::exit(1);
    }
}

async fn run_server() -> Result<(), &'static str> {
    let settings = Settings::from_env().map_err(|_| "application configuration is invalid")?;
    let repository = CassandraRepository::connect(&settings)
        .await
        .map_err(|_| "Cassandra connection or schema initialization failed")?;

    let address = settings.listen_address;
    let listener = TcpListener::bind(address)
        .await
        .map_err(|_| "HTTP listener could not bind")?;
    info!(%address, "URL shortener is listening");

    axum::serve(listener, api::router(ShortLinkService::new(repository)))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|_| "HTTP server stopped unexpectedly")
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut terminate = signal(SignalKind::terminate()).expect("install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = terminate.recv() => {},
        }
    }

    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}
