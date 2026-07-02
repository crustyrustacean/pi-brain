// dependencies
use pi_brain::configuration::get_configuration;
use pi_brain::database::{DatabaseBackend, SqliteRepository};
use pi_brain::startup::Application;
use pi_brain::telemetry::{get_subscriber, init_subscriber};
use std::sync::LazyLock;

// Ensure that the `tracing` stack is only initialised once.
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
    pub api_client: reqwest::Client,
    pub database: SqliteRepository,
}

pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");

        c.application.port = 0;
        c.database.path = ":memory:".to_string();
        c.database.max_connections = Some(1);

        c
    };

    // build the database backend
    let database = SqliteRepository::new(&configuration.database)
        .await
        .expect("Unable to build the database backend.");
    let database_for_test = database.clone();

    let database_backend: Box<dyn DatabaseBackend> = Box::new(database);

    // launch the application as a background task
    let application = Application::build(configuration.clone(), database_backend)
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    TestApp {
        address: format!("http://localhost:{}", application_port),
        port: application_port,
        api_client: client,
        database: database_for_test,
    }
}
