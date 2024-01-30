use log::{error, info, debug, trace};
use std::env;
use telnet::{Telnet, Event as TelnetEvent};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};


/// AMCP response struct
/// status_code: HTTP status
/// payload: AMCP response payload

struct AmcpResponse {
    status_code: u16,
    payload: String,
}


/// Send AMCP command to CasparCG server
/// command: AMCP command to send
/// host: CasparCG server hostname
/// port: CasparCG server port
///
/// Returns AmcpResponse struct

async fn send_amcp_command(command: String, host: &str, port: u16) -> AmcpResponse {
    // Connect to CasparCG server
    // If connection fails, return 502 Bad Gateway

    let mut connection = match Telnet::connect((host, port), 256) {
        Ok(conn) => conn,
        Err(_) => {
            error!("Failed to connect to {}:{}", host, port);
            return AmcpResponse { 
                status_code: 502, 
                payload: "Failed to connect to CasparCG server".to_string() 
            }
        }
    };

    // Append \r\n to command and send it to CasparCG server
    // If sending fails, return 502 Bad Gateway

    if let Err(_) = connection.write(format!("{}\r\n",command).as_bytes()) {
        error!("Failed to send AMCP command {} to {}:{}", command, host, port);
        return AmcpResponse { 
            status_code: 502, 
            payload: "Failed to send AMCP command to CasparCG server".to_string() 
        };
    }

    let mut response = String::new();
    let mut status_code: Option<u16> = None;

    loop {
        match connection.read_nonblocking() {
            Ok(TelnetEvent::Data(data)) => {
                response.push_str(&String::from_utf8_lossy(&data));

                trace!("Received data: {:?}", String::from_utf8_lossy(&data));

                // We don't know the status code yet, so try to parse it from the first line
                if status_code.is_none() && response.len() > 3 {
                    let parsed_code = response
                        .lines()
                        .next()
                        .unwrap_or("")
                        .split_whitespace()
                        .next();

                    if let Some(code) = parsed_code {
                        status_code = Some(code.parse::<u16>().unwrap_or(500));
                    }
                } 

                if response.ends_with("\r\n") {
                    if let Some(code) = status_code {
                        // If we receive a 201 OK, ensure we have the second part of the response
                        // if not, wait for it to arrive
                        if code == 201 && response.matches("\r\n").count() == 1 {
                            trace!("Received 201 OK, waiting for the rest of the response");
                            continue
                        }
                    }

                    // Other responses just end with \r\n, so we can break out of the loop
                    // and return the response. There may be more lines in the response, but
                    // we don't care about them.
                    trace!("Received full response {}", status_code.unwrap_or(500));
                    break;
                }


            },
            Ok(TelnetEvent::TimedOut) | Err(_) => break,
            _ => (),
        }
    }

    // If there are more lines in the response, return them as the payload
    // Otherwise, return the status line as the payload (e.g. "201 OK")

    debug!("Got {} response for AMCP command '{}'", status_code.unwrap_or(500), command);

    let lines: Vec<&str> = response.lines().collect();
    let payload = lines[1..].join("\n");

    AmcpResponse { status_code: status_code.unwrap_or(500), payload }
}


/// AMCP handler
/// body: AMCP command to send

async fn amcp_handler(body: String) -> impl Responder {
    let host = env::var("HTTP2AMCP_AMCP_HOST").unwrap_or("localhost".to_string());
    let port = env::var("HTTP2AMCP_AMCP_PORT").unwrap_or("5250".to_string()).parse::<u16>().unwrap_or(5250);

    let amcp_response = send_amcp_command(body, &host, port).await;
    let status_code = actix_web::http::StatusCode::from_u16(amcp_response.status_code).unwrap();

    HttpResponse::build(status_code)
        .content_type("text/plain")
        .body(amcp_response.payload)
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // Load settings from environment variables

    dotenv::dotenv().ok();

    let log_level = env::var("HTTP2AMCP_LOG_LEVEL").unwrap_or("info".to_string());
    let server_port = env::var("HTTP2AMCP_SERVER_PORT").unwrap_or("9731".to_string());

    // Create logger

    let level_filter = match log_level.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "warn" => log::LevelFilter::Warn,
        "warning" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };

    env_logger::Builder::from_default_env()
        .filter(None, level_filter)
        .init();

    // Start HTTP2AMCP server

    info!("Starting HTTP2AMCP server on port {}", server_port);

    HttpServer::new(|| { App::new().route("/amcp", web::post().to(amcp_handler)) })
        .bind(format!("0.0.0.0:{}", server_port))?
        .run()
        .await
}
