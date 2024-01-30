# HTTP2AMCP Proxy

This application acts as a proxy, translating HTTP POST requests 
into AMCP (Advanced Media Control Protocol) commands for CasparCG servers.

## Features

- Accepts HTTP POST requests and sends corresponding commands to a CasparCG server.
- Configurable through environment variables.
- Supports text/plain content type for both requests and responses.

## Configuration

The application can be configured using the following environment variables:

- `HTTP2AMCP_SERVER_PORT`: Port for the HTTP server to listen on (default: 9731).
- `HTTP2AMCP_AMCP_HOST`: Hostname or IP address of the CasparCG server (default: localhost).
- `HTTP2AMCP_AMCP_PORT`: Port of the CasparCG server (default: 5250).
- `HTTP2AMCP_LOG_LEVEL`: Log level (trace, debug, info, warn, error).

## Usage

To start the server, simply run the compiled binary. 
Ensure that the necessary environment variables are set or rely on the default values.


### HTTP POST Request

- Endpoint: `/amcp`
- Content-Type: text/plain
- Body: A single line of text representing the AMCP command to be sent to the CasparCG server.

Example:

```bash
http post http://localhost:9731/amcp --raw "PLAY 1-1 amb"
```

### Response

The response body (`text/plain`) contains the reply from the CasparCG server, which varies based on the executed command. 

It could be either empty (with the HTTP status code conveying the result) or contain text data (in case of a 200 OK response).

200 OK: The command was successfully executed, and the response (if any) from the AMCP server is returned in the body.

502 Bad Gateway: Indicates a proxy error, such as a failure to connect to the AMCP server.

Other HTTP status codes may be returned based on the AMCP server's response, reflecting various error or informational states.

## Building the Application

Ensure you have Rust and Cargo installed. Then run:

```bash
cargo build --release
```

### Running as a systemd service

Copy the binary to /usr/local/bin/http2amcp

Create a file named `/etc/systemd/system/http2amcp.service` with the following contents:

```ini
[Unit]
Description=HTTP2AMCP Proxy Service
After=network.target

[Service]
WorkingDirectory=/usr/local/bin
ExecStart=/usr/local/bin/http2amcp
Environment="HTTP2AMCP_SERVER_PORT=9731"
Environment="HTTP2AMCP_AMCP_HOST=localhost"
Environment="HTTP2AMCP_AMCP_PORT=5250"
Environment="HTTP2AMCP_LOG_LEVEL=info"
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

