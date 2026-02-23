![icon.png](https://bimply.lifelike.dev/d/ZyRdaH55Au)

# MC-HA

> Minecraft Headless Authenticator for 1.21.2+

A simple way to authenticate a minecraft account via joining a server & syncing a token.  

## Usage

```rust
use mc_headless_auth::{Server, ServerConfig};

fn main() {
    let server = Server::start(ServerConfig::default());

    let token = String::from("...");
    let result = server.verify(&token.trim().to_uppercase());
    assert!(result.is_some());
}
```

## Versions

Due to network protocol changes, currently the library only supports `1.21.2+` *(`768`)*.  
This has been tested to work in `1.21.2`, `1.21.3`, `1.21.4`, `1.21.5`, `1.21.6`, `1.21.8`, `1.21.9`, `1.21.10` & `1.21.11`.  
Technically the networking protocol and authentication system should allow for any client with a version of `1.7+` to work.  
But I've encountered plenty of small differences in networking between that and now that isn't really documented anywhere.  

### Non-working versions

Versions that I've tested to **not** work and their vague reasoning on why.  

- **1.21.1**, **1.21** `Failed to decode clientbound/minecraft:game_profile`  
- **1.20**, **1.19.2**, **1.19**, **1.16.1**, **1.12.2**, **1.8** `Found 1 bytes extra whilst reading packet 1`
- **1.15** `<No Error>`
- **1.7.2** `Key was smaller than nothing! Weird key!`

Note that these are from the latest version of this library as of writing this.  
In the past I have gotten different errors than what is written here.  
Like `1.16.1` have complained about extra bytes in a different sequence.  
And `1.12.2` and `1.8` have responded with `IndexOutOfBounds` errors in the past.


## Server List Pings

The server implements legacy pings for all previous versions, including:  
- `Beta 1.8` to `1.3`  
- `1.4` to `1.5`
- `1.6`

And of course the modern server ping sequence.  

## Configuration

Everything that the client visually sees can be customized and how the token generates can also be overriden.  

### Server Configuration

A `Token` and `Message` generator can be supplied when starting a minecraft server.  
The `Token` generator takes in the players `username` and `uuid` and must return a string that the client receives.  
This token is then synced with the player to verify it later on.  
The `Message` generator takes in the token and must return a valid [text component](https://minecraft.wiki/w/Text_component_format) that is displayed to the user upon a successful disconnection.  

For more look at the `ServerConfig`.  


### Status Configuration

The `favicon` for server list ping can changed to any `64x64` `png` image.  
The `description` displayed in the server list can be any valid `Nbt` [text component](https://minecraft.wiki/w/Text_component_format).  
You can also supply a `legacy_decription` which is just a simple string that is used in legacy ping packets,  
and also if a client is too old to join.  