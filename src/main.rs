use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use z2m::configuration::get_configuration;
use z2m::startup::run;
use z2m::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("z2m".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address)?;
    let connection_pool =
        PgPoolOptions::new().connect_lazy_with(configuration.database.with_db());

    run(listener, connection_pool)?.await
}
