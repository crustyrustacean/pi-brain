// tests/api/helpers.rs

// dependencies
use knowledge_base::configuration::{get_configuration, DatabaseSettings};
use knowledge_base::startup::{get_connection_pool, Application};
use knowledge_base::telemetry::{get_subscriber, init_subscriber};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::sync::LazyLock;
use std::str::FromStr;
use tempfile::NamedTempFile;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

#[allow(dead_code)]
pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: SqlitePool,
    pub api_client: reqwest::Client,
    // Keep the temp file alive for the test duration
    _temp_db: NamedTempFile,
}

pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    // Create a temporary SQLite database file
    let temp_db = NamedTempFile::new().expect("Failed to create temp database file");
    let db_path = temp_db.path().to_string_lossy().to_string();

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.path = db_path;
        c.application.port = 0;
        c
    };

    configure_database(&configuration.database).await;

    // Launch the application as a background task
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let test_app = TestApp {
        address: format!("http://localhost:{}", application_port),
        port: application_port,
        db_pool: get_connection_pool(&configuration.database),
        api_client: client,
        _temp_db: temp_db,
    };

    test_app
}

async fn configure_database(config: &DatabaseSettings) -> SqlitePool {
    // Create and migrate database
    let connection_string = config.connection_string();
    let options = SqliteConnectOptions::from_str(&connection_string)
        .expect("Failed to parse connection string")
        .create_if_missing(true);
    
    let connection_pool = SqlitePool::connect_with(options)
        .await
        .expect("Failed to connect to SQLite.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    
    connection_pool
}
