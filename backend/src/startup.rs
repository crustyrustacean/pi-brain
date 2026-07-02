// src/startup.rs

// dependencies
use crate::configuration::Settings;
use crate::database::DatabaseBackend;
use crate::routes::{
    create_document, delete_document, get_document, get_endpoints, get_stats, health_check,
    list_documents, search_documents, search_get, update_document,
};
use actix_cors::Cors;
use actix_files::Files;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web, web::Data};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(
        configuration: Settings,
        database: Box<dyn DatabaseBackend>,
    ) -> Result<Self, anyhow::Error> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr()?.port();
        let server = run(listener, database, configuration.application.base_url).await?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub struct ApplicationBaseUrl(pub String);

async fn run(
    listener: TcpListener,
    database: Box<dyn DatabaseBackend>,
    base_url: String,
) -> Result<Server, anyhow::Error> {
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let database = Data::new(database);

    let server = HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .service(create_document)
            .service(get_document)
            .service(update_document)
            .service(delete_document)
            .service(list_documents)
            .service(search_documents)
            .service(search_get)
            .service(get_stats)
            .route("/kb/endpoints", web::get().to(get_endpoints))
            .service(Files::new("/", "../frontend/dist").index_file("index.html"))
            .app_data(base_url.clone())
            .app_data(database.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
