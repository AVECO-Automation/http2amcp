use log::{error, info, debug};
use std::env;
use telnet::{Telnet, Event as TelnetEvent};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};


struct AmcpResponse {
    status_code: u16,
    payload: Option<String>,
}


async fn send_amcp_command(command: String, host: &str, port: u16) -> AmcpResponse {
    let mut connection = match Telnet::connect((host, port), 256) {
        Ok(conn) => conn,
        Err(_) => {
            error!("Failed to connect to {}:{}", host, port);
            return AmcpResponse { status_code: 502, payload: None }
        }
    };

    let command = format!("{}\r\n", command);
    if let Err(_) = connection.write(command.as_bytes()) {
        error!("Failed to send AMCP command {} to {}:{}", command, host, port);
        return AmcpResponse { status_code: 502, payload: None };
    }

    let mut response = String::new();
    loop {
        match connection.read_nonblocking() {
            Ok(TelnetEvent::Data(data)) => {
                response.push_str(&String::from_utf8_lossy(&data));
                if response.ends_with("\r\n\r\n") || response.ends_with("\r\n") {
                    break;
                }
            },
            Ok(TelnetEvent::TimedOut) | Err(_) => break,
            _ => (),
        }
    }

    let status_line = response.lines().next().unwrap_or("");
    let status_code = status_line.split_whitespace().next().unwrap_or("500").parse::<u16>().unwrap_or(500);

    // If there are more lines in the response, return them as the payload
    // Otherwise, return the status line as the payload (e.g. "201 OK")

    debug!("Executed AMCP command {}", command);

    let payload = if response.lines().count() > 1 {
        Some(response.lines().skip(1).collect::<Vec<&str>>().join("\r\n"))
    } else {
        Some(status_line.to_string())
    };

    AmcpResponse { status_code, payload }
}


async fn amcp_handler(body: String) -> impl Responder {
    let host = env::var("HTTP2AMCP_AMCP_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("HTTP2AMCP_AMCP_PORT").unwrap_or_else(|_| "5250".to_string()).parse().unwrap_or(5250);

    let amcp_response = send_amcp_command(body, &host, port).await;
    HttpResponse::build(actix_web::http::StatusCode::from_u16(amcp_response.status_code).unwrap())
                .content_type("text/plain")
                .body(amcp_response.payload.unwrap_or_default())
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // get log level from HTTP2AMCP_LOG_LEVEL env var

    let log_level = env::var("HTTP2AMCP_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let level_filter = match log_level.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };

    env_logger::Builder::from_default_env()
    .filter(None, level_filter)
    .init();

    let server_port = env::var("HTTP2AMCP_SERVER_PORT").unwrap_or_else(|_| "9731".to_string());

    info!("Starting HTTP2AMCP server on port {}", server_port);
    HttpServer::new(|| {
        App::new()
            .route("/amcp", web::post().to(amcp_handler))
    })
    .bind(format!("0.0.0.0:{}", server_port))?
    .run()
    .await
}
