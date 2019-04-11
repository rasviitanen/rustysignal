use web_push::*;

use std::{
    fs::File,
    io::Read,
    time::Duration,
};

use futures::{
    future::{
        lazy,
    },
    Future,
};

pub fn push(push_payload: &str, subscription: &str) {
    println!("!!!!!! Sending PUSH !!!!!!!");
    println!("{:?}", subscription);

    let subscription_info: SubscriptionInfo = serde_json::from_str(subscription).unwrap();

    let mut builder = WebPushMessageBuilder::new(&subscription_info).unwrap();

    builder.set_payload(ContentEncoding::AesGcm, push_payload.as_bytes());

    let file = File::open("cert/vapid/private.pem").unwrap();
    
    let sig_builder = VapidSignatureBuilder::from_pem(file, &subscription_info).unwrap();
    let signature = sig_builder.build().unwrap();

    builder.set_ttl(3600);
    builder.set_vapid_signature(signature);

    match builder.build() {
        Ok(message) => {
            let client = WebPushClient::new().unwrap();
            tokio::run(lazy(move || {
                client
                    .send_with_timeout(message, Duration::from_secs(4))
                    .map(|response| {
                        println!("Sent: {:?}", response);
                    }).map_err(|error| {
                        println!("Error: {:?}", error)
                    })
            }));
        },
        Err(error) => {
            println!("ERROR in building message: {:?}", error)
        }
    }
}
