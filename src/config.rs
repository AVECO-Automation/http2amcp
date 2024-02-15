use clap::Parser;
use lazy_static::lazy_static;
use dotenv::dotenv;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[clap(
    name = "http2amcp",
    about = "HTTP to AMCP bridge",
    version = env!("CARGO_PKG_VERSION"),
)]
pub struct Config {

    #[clap(long, env = "HTTP2AMCP_AMCP_HOST", default_value = "localhost")]
    pub host: String,

    #[clap(long, env = "HTTP2AMCP_AMCP_PORT", default_value = "5250")]
    pub port: u16,

    #[clap(long, env = "HTTP2AMCP_SERVER_PORT", default_value = "9731")]
    pub server_port: u16,

    #[clap(long, env = "HTTP2AMCP_LOG_LEVEL", default_value = "info")]
    pub log_level: String,
}


lazy_static! {
    pub static ref CONFIG: Config = {
        dotenv().ok(); // Load .env file if it exists
        Config::parse()
    };
}
