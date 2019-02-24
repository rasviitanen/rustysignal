use std::str;
use std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;
use std::collections::HashMap;

use ws::{listen, Handler, Sender, Result, Message, Handshake, CloseCode};
use serde_json::Value;

use push;

#[derive(Default)]
struct Network {
    nodemap: Rc<RefCell<HashMap<String, Weak<RefCell<Node>>>>>,
    pushmap: Rc<RefCell<HashMap<String, String>>>,
}

impl Network {
    fn add_user(&mut self, owner: &str, node: &std::rc::Rc<std::cell::RefCell<Node>>) {
        if !self.nodemap.borrow().contains_key(owner) {
            node.borrow_mut().owner = Some(owner.into());
            self.nodemap.borrow_mut().insert(owner.to_string(), Rc::downgrade(node));
            println!("Node {:?} connected to the network.", owner);
        } else {
            println!("{:?} tried to connect, but the username was taken", owner);
            node.borrow().sender.send("The username is taken").ok();
        }
    }

    fn add_subscription(&mut self, subscription: &str, node: &std::rc::Rc<std::cell::RefCell<Node>>) {
        println!("Node {:?} updated its subscription data", node.borrow().owner);
        //node.borrow_mut().subscription = Some(subscription.into());
        let owner = node.borrow().owner.clone();
        self.pushmap.borrow_mut().insert(owner.unwrap(), subscription.to_string());

    }

    fn remove(&mut self, owner: &str) {
        self.nodemap.borrow_mut().remove(owner);
    }

    fn size(&self) -> usize {
        self.nodemap.borrow().len()
    }
}

struct Node {
    owner: Option<String>,
    subscription: Option<String>,
    sender: Sender
}

struct Server {
    node: Rc<RefCell<Node>>,
    network: Rc<RefCell<Network>>,
}

impl Handler for Server {
    fn on_open(&mut self, handshake: Handshake) -> Result<()> {
        // Get the aruments from a URL
        // i.e localhost:8000/user=testuser
        let url_arguments = handshake.request.resource()[2..].split("=");
        // Beeing greedy by not collecting pairs
        // Instead every even number (including 0) will be an identifier
        // and every odd number will be the assigned value
        let argument_vector: Vec<&str> = url_arguments.collect();

        if argument_vector[0] == "user" {
            let username: &str = argument_vector[1];
            self.network.borrow_mut().add_user(username, &self.node);
        }

        println!("Network expanded to {:?} connected nodes\n", self.network.borrow().size());
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let sender_msg_string: &str = msg.as_text()?;
        let json_message: Value = 
            serde_json::from_str(sender_msg_string).unwrap_or(Value::default());
        
        match json_message["action"].as_str() {
            Some("subscribe") => { 
                    match json_message["data"].as_str() {
                         Some(data) => {
                             self.network.borrow_mut().add_subscription(data, &self.node);
                         },
                         _ => { println!("No subscription data") }
                    }
                },
            Some("connection_request") => {
                match json_message["connect_to"].as_str() {
                    Some(receiver) => {
                        let network = self.network.borrow();
                        let user_sending_request = self.node.borrow().owner.clone().unwrap();

                        let payload = json!({"body": format!("{}\nwants to connect with you", user_sending_request), "sender": user_sending_request, "actions": [{"action": "AllowConnection", "title": "✔️ Allow"}, {"action": "DenyConnection", "title": "✖️ Deny"}]});

                        network.pushmap.borrow().get(receiver).and_then(
                            |sub| Some(push::push(&payload.to_string(), &sub))
                        );
                    }
                    _ => {}
                }
            },
            _ => {
                // !!! WARNING !!!
                // The word "protocol" match is protcol specific.
                // Thus a client should make sure to send a viable protocol
                let protocol = match json_message["protocol"].as_str() {
                    Some(desired_protocol) => { Some(desired_protocol) },
                    _ => { None }
                };

                // The words below are protcol specific.
                // Thus a client should make sure to use a viable protocol
                match protocol {
                    Some("one-to-all") => {
                        self.node.borrow().sender.broadcast(sender_msg_string);
                    },
                    Some("one-to-self") => {
                        self.node.borrow().sender.send(sender_msg_string);
                    },
                    Some("one-to-one") => {
                        match json_message["to"].as_str() {
                            Some(receiver) => {
                                let network = self.network.borrow();
                                let receiver_node = network.nodemap.borrow().get(receiver)
                                    .and_then(|node| node.upgrade());

                                match receiver_node {
                                    Some(node) => { node.borrow().sender.send(sender_msg_string); }
                                    _ => {self.node.borrow().sender
                                        .send("Could not find a node with that name");}
                                };
                            }
                            _ => {
                                self.node.borrow().sender.send(
                                    "No field 'to' provided"
                                );
                            }
                        }
                        
                    }
                    _ => {
                        self.node.borrow().sender.send(
                                "Invalid protocol, valid protocols include: 
                                    'one-to-one'
                                    'one-to-many'
                                    'one-to-all'"
                            );
                        }
                }
            }
        };
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        // Remove the node from the network
        if let Some(owner) = &self.node.borrow().owner {
            match code {
                CloseCode::Normal =>
                    println!("{:?} is done with the connection.", owner),
                CloseCode::Away =>
                    println!("{:?} left the site.", owner),
                CloseCode::Abnormal =>
                    println!("Closing handshake for {:?} failed!", owner),
                _ =>
                    println!("{:?} encountered an error: {:?}", owner, reason),
            };
        
            self.network.borrow_mut().remove(owner)
        }
        
        println!("Network shrinked to {:?} connected nodes\n", self.network.borrow().size());
    }

    fn on_error(&mut self, err: ws::Error) {
        println!("The server encountered an error: {:?}", err);
    }
}

pub fn run() {
    let network = Rc::new(RefCell::new(Network::default()));

    // Setup logging
    env_logger::init();

    // setup command line arguments
    let matches = clap::App::new("Rustysignal")
        .version("2.0.0")
        .author("Rasmus Viitanen <rasviitanen@gmail.com>")
        .about("A signaling server implemented in Rust that can be used for e.g. WebRTC, see https://github.com/rasviitanen/rustysignal. To use SSL, use rustysignal --features ssl")
        .arg(
            clap::Arg::with_name("ADDR")
                .help("Address on which to bind the server e.g. 127.0.0.1:3012")
                .required(true)
                .index(1),
        )
        .get_matches();

    
    println!("------------------------------------");
    println!("rustysignal is listening on address");
    println!("ws://{}", matches.value_of("ADDR").unwrap());
    println!("To use SSL you need to reinstall rustysignal using 'cargo install rustysignal --features ssl --force");
    println!("-------------------------------------");
    
    listen(matches.value_of("ADDR").unwrap(),
        |sender| {
            let node = Node { owner: None, subscription: None, sender };
            Server { 
                node: Rc::new(RefCell::new(node)),
                network: network.clone()
            }
        }
    ).unwrap()
}