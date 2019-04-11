extern crate ws;
#[macro_use]
extern crate serde_json;

extern crate clap;
extern crate env_logger;

extern crate serde;
extern crate tokio;
extern crate base64;
extern crate futures;

#[cfg(feature = "ssl")]
extern crate openssl;
#[cfg(feature = "push")]
extern crate web_push;

mod server;

mod node;
mod network;

fn main() {
    server::run()
}