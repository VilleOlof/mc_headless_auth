use std::io;

use mc_headless_auth::{Server, ServerConfig};

fn main() {
    // TODO: Make this a more concrete example where this also hosts a small website
    // that takes in a token and responds with the uuid & username if the token exists to showcase.

    let config = ServerConfig::default();
    let server = Server::start(config.clone());
    println!("Started server on localhost:{}", config.port);

    server.on_error(|e| {
        println!("{e:?}");
    });

    server.on_join(|user, _| {
        println!("{} joined the server", user.username);
    });

    println!("Enter the token given by the server");
    let mut token = String::new();
    io::stdin().read_line(&mut token).unwrap();

    let result = server.verify(&token.trim().to_uppercase());

    println!("{result:#?}");
}
