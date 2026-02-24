use mc_headless_auth::{Server, ServerConfig};

fn main() {
    let config = ServerConfig::default();
    let server = Server::start(config.clone());
    println!("Started server on '0.0.0.0:{}'", config.port);

    server.on_error(|e| {
        eprintln!("{e:?}");
    });

    server.on_join(|user, _| {
        println!(
            "'{}' joined the server ({})",
            user.username,
            user.uuid.as_hyphenated()
        );
    });

    std::io::stdin().read_line(&mut String::new()).unwrap();
}
