// src/main.rs

// dependencies
use knowledge_base::configuration::get_configuration;
use knowledge_base::startup::Application;
use knowledge_base::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("knowledge-base".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration.clone()).await?;
    application.run_until_stopped().await?;

    Ok(())
}
