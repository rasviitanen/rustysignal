extern crate ws;
extern crate serde_json;

use std::str;
use std::rc::Rc;
use std::rc::Weak;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;

use ws::{listen, Handler, Sender, Result, Message, Handshake, CloseCode};
use serde_json::Value;

#[derive(Default)]
struct Network {
    nodes: Rc<RefCell<Vec<Weak<RefCell<Node>>>>>,
    nodemap: Rc<RefCell<HashMap<String, usize>>>,
}

impl Network {
    fn add_node(&mut self, node: Node) -> Rc<RefCell<Node>> {
        let node = Rc::new(RefCell::new(node));
        self.nodes.borrow_mut().push(Rc::downgrade(&node));
        node
    }
    fn assign_user(&mut self, username: String) {
        self.nodemap.borrow_mut().insert(username, self.nodes.borrow().len()-1);
    }
    fn index_of_user(&self, username: &str) -> Option<usize> {
        self.nodemap.borrow().get(username)
            .and_then(|index|{ Some(index.clone()) })
    }
    fn connection_id(&self, index: usize) -> Option<u32> {
        self.nodes.borrow().get(index)
            .and_then(|node_refcell| { node_refcell.upgrade() }
            .and_then(|node| { Some(node.borrow().sender.connection_id())}))
    }
    fn token(&self, index: usize) -> Option<ws::util::Token> {
        self.nodes.borrow().get(index)
            .and_then(|x| { x.upgrade() }
            .and_then(|x| { Some(x.borrow().sender.token()) }))
    }
}

struct Node {
    owner: Option<String>,
    sender: Sender
}

struct Server {
    node: Rc<RefCell<Node>>,
    count: Rc<Cell<u32>>,
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
            self.node.borrow_mut().owner = Some(username.into());
            self.network.borrow_mut().assign_user(username.into());
        }
        Ok(self.count.set(self.count.get() + 1))
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
                        let receiver_index = self.network.borrow().index_of_user(&receiver);
                        match receiver_index {
                            Some(index) => {
                                self.node.borrow().sender.send_to(
                                    self.network.borrow().connection_id(index).unwrap(),
                                    self.network.borrow().token(index).unwrap(),
                                    sender_msg_string
                                )
                            },
                            _ => {
                                self.node.borrow().sender.send(
                                    "No node with that name"
                                )
                            }
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
        match code {
            CloseCode::Normal =>
                println!("The client is done with the connection."),
            CloseCode::Away =>
                println!("The client is leaving the site."),
            CloseCode::Abnormal =>
                println!("Closing handshake failed!"),
            _ =>
                println!("The client encountered an error: {}", reason),
        };
        self.count.set(self.count.get() - 1)
    }

    fn on_error(&mut self, err: ws::Error) {
        println!("The server encountered an error: {:?}", err);
    }
}

fn main() {
    let count = Rc::new(Cell::new(0));
    let network = Rc::new(RefCell::new(Network::default()));

    listen("127.0.0.1:3012",
        |sender| {
            let node = Node { owner: None, sender };
            let _node = network.borrow_mut().add_node(node);
            Server { 
                node: _node,
                count: count.clone(),
                network: network.clone()
            }
        }
    ).unwrap()
} 