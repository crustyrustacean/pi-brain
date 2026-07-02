// src/main.rs

// dependencies
use pi_brain::configuration::get_configuration;
use pi_brain::database::{DatabaseBackend, SqliteRepository};
use pi_brain::startup::Application;
use pi_brain::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    let subscriber = get_subscriber("pi-brain".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // build the configuration
    let configuration = get_configuration().expect("Failed to read configuration.");

    // build the database backend
    let database: Box<dyn DatabaseBackend> =
        Box::new(SqliteRepository::new(&configuration.database).await?);

    // build the application by passing the configuration and database backend
    let application = Application::build(configuration.clone(), database).await?;

    // run the application
    application.run_until_stopped().await?;

    Ok(())
}
