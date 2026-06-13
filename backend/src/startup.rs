// src/startup.rs

// dependencies
use crate::configuration::{DatabaseSettings, Settings};
use crate::routes::{documents, endpoints, health_check, search, stats};
use actix_files::Files;
use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web, web::Data};
use sqlx::sqlite::{SqlitePoolOptions, SqliteConnectOptions};
use sqlx::SqlitePool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use std::str::FromStr;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr()?.port();
        let server = run(listener, connection_pool, configuration.application.base_url).await?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> SqlitePool {
    SqlitePoolOptions::new().connect_lazy_with(
        SqliteConnectOptions::from_str(&configuration.connection_string()).unwrap()
            .create_if_missing(true)
    )
}


pub struct ApplicationBaseUrl(pub String);

async fn run(listener: TcpListener, db_pool: SqlitePool, base_url: String) -> Result<Server, anyhow::Error> {
    // Run migrations
    sqlx::migrate!().run(&db_pool).await?;
    
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .service(documents::create_document)
            .service(documents::get_document)
            .service(documents::update_document)
            .service(documents::delete_document)
            .service(documents::list_documents)
            .service(search::search_documents)
            .service(search::search_get)
            .service(stats::get_stats)
            .route("/kb/endpoints", web::get().to(endpoints::get_endpoints))
            .service(Files::new("/", "../frontend/dist").index_file("index.html"))
            .app_data(base_url.clone())
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
