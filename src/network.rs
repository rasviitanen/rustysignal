use std::str;
use std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;
use std::collections::HashMap;
#[cfg(feature = "push")]
use std::{
    fs::File,
    time::Duration,
};

#[cfg(feature = "push")]
use futures::{
    future::{
        lazy,
    },
    Future,
};
#[cfg(feature = "push")]
use web_push::*;

use node::Node;

#[cfg(feature = "push")]
#[derive(Default)]
pub struct Network {
    pub nodemap: Rc<RefCell<HashMap<String, Weak<RefCell<Node>>>>>,
    pub pushmap: Rc<RefCell<HashMap<String, String>>>,
    pub vapid_path: String,
}

#[cfg(not(feature = "push"))]
#[derive(Default)]
pub struct Network {
    pub nodemap: Rc<RefCell<HashMap<String, Weak<RefCell<Node>>>>>,
}

impl Network {
    pub fn add_user(&mut self, owner: &str, node: &std::rc::Rc<std::cell::RefCell<Node>>) {
        if !self.nodemap.borrow().contains_key(owner) {
            node.borrow_mut().owner = Some(owner.into());
            self.nodemap.borrow_mut().insert(owner.to_string(), Rc::downgrade(node));
            println!("Node {:?} connected to the network.", owner);
        } else {
            println!("{:?} tried to connect, but the username was taken", owner);
            node.borrow().sender.send("The username is taken").ok();
        }
    }

    pub fn remove(&mut self, owner: &str) {
        self.nodemap.borrow_mut().remove(owner);
    }

    pub fn size(&self) -> usize {
        self.nodemap.borrow().len()
    }
    
    #[cfg(feature = "push")]
    pub fn add_subscription(&mut self, subscription: &str, node: &std::rc::Rc<std::cell::RefCell<Node>>) {
        println!("Node {:?} updated its subscription data", node.borrow().owner);
        node.borrow_mut().subscription = Some(subscription.into());
        let owner = node.borrow().owner.clone();
        
        self.pushmap.borrow_mut().insert(owner.unwrap(), subscription.to_string());
    }

    #[cfg(feature = "push")]
    pub fn set_vapid_path(&mut self, vapid_path: &str) {
        self.vapid_path = vapid_path.to_string();
    }

    #[cfg(feature = "push")]
    pub fn send_push(&self, sender: &str, endpoint: &str) {
        println!("!!!!!! Sending PUSH !!!!!!!");

        let payload = 
            json!({"body": format!("{}\nwants to connect with you", sender), 
            "sender": sender, 
            "actions": [
                {"action": "allowConnection", "title": "✔️ Allow"}, 
                {"action": "denyConnection", "title": "✖️ Deny"}]}).to_string();

        if let Some(subscription) = self.pushmap.borrow().get(endpoint) {
            let subscription_info: SubscriptionInfo = serde_json::from_str(subscription).unwrap();

            let mut builder = WebPushMessageBuilder::new(&subscription_info).unwrap();
            builder.set_payload(ContentEncoding::AesGcm, payload.as_bytes());

            let vapid_file = File::open(&self.vapid_path).unwrap();

            let sig_builder = VapidSignatureBuilder::from_pem(vapid_file, &subscription_info).unwrap();
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
    }
}