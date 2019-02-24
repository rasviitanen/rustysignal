extern crate ws;
#[macro_use]
extern crate serde_json;

extern crate clap;
extern crate env_logger;

extern crate serde;
extern crate web_push;
extern crate tokio;
extern crate base64;
extern crate futures;

#[cfg(feature = "ssl")]
extern crate openssl;

#[cfg(not(feature = "ssl"))]
mod regularserver;
#[cfg(feature = "ssl")]
mod sslserver;

mod push;

#[cfg(not(feature = "ssl"))]
fn main() {
    regularserver::run()
}

#[cfg(feature = "ssl")]
fn main() {
    sslserver::run()
}