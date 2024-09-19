use std::net::TcpListener;
use sqlx::{PgConnection, Connection};
use z2m::configuration::get_configuration;

#[tokio::test]
async fn test_health_check() {
    let address = spawn_app();
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Can't send request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app();
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string).await.expect("Failed to connect to Postgres");
    let client = reqwest::Client::new();

    let body = "name=hwait&email=hwaitt@gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "hwaitt@gmail.com");
    assert_eq!(saved.name, "hwait");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=hwaitt", "missing the email"),
        ("email=hwaitt@gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (body, error_message) in test_cases {
        let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

        assert_eq!(400, response.status().as_u16(), "The API did not fail with 400 Bad Request when the payload was {}.", error_message);
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = z2m::startup::run(listener).expect("Failed to obtain Server");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}