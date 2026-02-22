<div style="display:flex; align-items: center; gap: 1rem;">
    <img src="icon.png">
    <h1>MC-HA</h1>
</div>

---

> Minecraft Headless Authenticator for 1.7+

a simple way to authenticate a minecraft account via joining a server & retrieving a token.  

## Steps

- Minecraft account joins the server
- The server logs their uuid and generates a token for that account & kicks the player with that token as a response
- Then that token can be used to link that uuid to whatever account logic or further processing



## Lib

Should both be a binary and a library, the binary should just start the server and print the uuid & token together in stdout.  

The library should expose functions like:

starting the server should spawn a new thread and run the server  
and returns a server struct that internally holds communication to the server
and can verify arbritaty tokens and see of they fit an account
```rust
let server = start_server().await;
```

```rust
// important that the server is clonable and have an internal Arc for the channel communication etc.
// so the user can just clone it and hold the same connection to the started server for ease of use
#[derive(Clone)]
struct Server {
    on_join: Channel,
    verify: Channel
}

impl Server {
    pub fn verify(token: impl AsRef<str>) -> Result<User> {}
}

struct User {
    uuid: Uuid,
    username: String,
    ...
}
```

## Usage

```rust
use mc_headless_auth::{Server, ServerConfig};

fn main() {
    let server = Server::start(ServerConfig::default());

    let token = String::from("...");
    let result = server.verify(&token.trim().to_uppercase());
    assert!(result.is_ok());
}
```