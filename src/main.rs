use std::io;

use mc_headless_auth::{Server, ServerConfig};

fn main() {
    let config = ServerConfig::default();
    let server = Server::start(config.clone());
    println!("Started server on localhost:{}", config.port);

    println!("Enter the token given by the server");
    let mut token = String::new();
    io::stdin().read_line(&mut token).unwrap();

    let result = server.verify(&token.trim().to_uppercase());

    println!("{result:#?}");
}
