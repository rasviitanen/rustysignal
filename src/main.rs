extern crate ws;
extern crate serde_json;

extern crate clap;
extern crate env_logger;

#[cfg(feature = "ssl")]
extern crate openssl;

#[cfg(not(feature = "ssl"))]
mod regularserver;
#[cfg(feature = "ssl")]
mod sslserver;

#[cfg(not(feature = "ssl"))]
fn main() {
    regularserver::run()
}

#[cfg(feature = "ssl")]
fn main() {
    sslserver::run()
}