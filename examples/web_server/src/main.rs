use std::str::FromStr;

use mc_headless_auth::ServerConfig;
use serde_json::json;
use tiny_http::{Header, Response};

fn main() {
    let config = ServerConfig::default();
    let mc_server = mc_headless_auth::Server::start(config.clone());
    println!(
        "> mc: started minecraft server on 'localhost:{}'",
        config.port
    );

    mc_server.on_error(|e| {
        eprintln!("> mc: server error: {e:?}");
    });

    mc_server.on_join(|user, token| {
        println!("> mc: '{}' joined ({})", user.username, token);
    });

    let http_server = tiny_http::Server::http("0.0.0.0:8000").unwrap();
    println!("> http: started web server on 'http://localhost:8000'");

    for mut request in http_server.incoming_requests() {
        let mc = mc_server.clone();

        println!("> http: [{}] {}", request.method().as_str(), request.url());

        match request.url() {
            "/" => {
                request
                    .respond(
                        Response::from_data(include_bytes!("../index.html"))
                            .with_header(Header::from_str("Content-Type:text/html").unwrap()),
                    )
                    .unwrap();
            }
            "/verify" => {
                let mut user_token = String::with_capacity(10);
                request.as_reader().read_to_string(&mut user_token).unwrap();
                println!("  Verifying {}", user_token);

                match mc.verify(user_token) {
                    Some(u) => {
                        let json = serde_json::to_string(&json!({
                            "username": u.username,
                            "uuid": u.uuid
                        }))
                        .unwrap();

                        request.respond(Response::from_string(json)).unwrap();
                    }
                    None => request.respond(Response::empty(404)).unwrap(),
                };
            }
            _ => request.respond(Response::empty(404)).unwrap(),
        }
    }
}
