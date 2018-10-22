extern crate ws;
extern crate serde_json;

use std::str;
use std::env;
use std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;
use std::collections::HashMap;

use ws::{listen, Handler, Sender, Result, Message, Handshake, CloseCode};
use serde_json::Value;

#[derive(Default)]
struct Network {
    nodemap: Rc<RefCell<HashMap<String, Weak<RefCell<Node>>>>>,
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

    fn remove(&mut self, owner: &str) {
        self.nodemap.borrow_mut().remove(owner);
    }

    fn size(&self) -> usize {
        self.nodemap.borrow().len()
    }
}

struct Node {
    owner: Option<String>,
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
                self.node.borrow().sender.broadcast(sender_msg_string)
            },
            Some("one-to-self") => {
                self.node.borrow().sender.send(sender_msg_string)
            },
            Some("one-to-one") => {
                match json_message["to"].as_str() {
                    Some(receiver) => {
                        let network = self.network.borrow();
                        let receiver_node = network.nodemap.borrow().get(receiver)
                            .and_then(|node| node.upgrade());

                        match receiver_node {
                            Some(node) => {node.borrow().sender.send(sender_msg_string)}
                            _ => self.node.borrow().sender
                                .send("Could not find a node with that name")
                        }
                    }
                    _ => {
                        self.node.borrow().sender.send(
                            "No field 'to' provided"
                        )
                    }
                }
                
            }
            _ => {
                self.node.borrow().sender.send(
                        "Invalid protocol, valid protocols include: 
                            'one-to-one'
                            'one-to-many'
                            'one-to-all'"
                    )
                }
        }
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

fn main() {
    let network = Rc::new(RefCell::new(Network::default()));

    // Allow to start the server on other address by suppling an argument
    // e.g. cargo run 127.0.0.1:3015, default to 127.0.0.1:3012
    let address: String;
    let args: Vec<String> = env::args().collect();    
    if let Some(address_arg) = args.get(1) {
        address = address_arg.to_string();
    } else {
        address = "127.0.0.1:3012".to_string();
    }

    println!("------------------------------------");
    println!("rustysignal is listening on socket address:\n{:?}", address);
    println!("-------------------------------------");
    
    listen(address,
        |sender| {
            let node = Node { owner: None, sender };
            Server { 
                node: Rc::new(RefCell::new(node)),
                network: network.clone()
            }
        }
    ).unwrap()
} 