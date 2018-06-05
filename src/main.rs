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
    fn add_username(&mut self, username: String) {
        self.nodemap.borrow_mut().insert(username, self.nodes.borrow().len());
    }
    fn index_of(&self, username: &str) -> Option<usize> {
        self.nodemap.borrow().get(username).and_then(|x|{ Some(x.clone() - 1) })
    }
    fn connection_id(&self, index: usize) -> Option<u32> {
        self.nodes.borrow().get(index).and_then(|x| {x.upgrade()}.and_then(|x| {Some(x.borrow().sender.connection_id())}))
    }
    fn token(&self, index: usize) -> Option<ws::util::Token> {
        self.nodes.borrow().get(index).and_then(|x| {x.upgrade()}.and_then(|x| {Some(x.borrow().sender.token())}))
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
        let arguments = handshake.request.resource()[2..].split("=");
        let argument_vector: Vec<&str> = arguments.collect();
        let username: &str = argument_vector[1];

        self.node.borrow_mut().owner = Some(username.into());
        self.network.borrow_mut().add_username(username.into());
        Ok(self.count.set(self.count.get() + 1))
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let msg_string: &str = msg.as_text()?;
        // WARNING: PROTOCOL SPECIFIC
        let json_message: Value = serde_json::from_str(msg_string).unwrap();
        let send_to = json_message["to"].as_str().unwrap();
        let sender_index = self.network.borrow().index_of(send_to);

        match sender_index {
            Some(index) => {
                self.node.borrow().sender.send_to(
                        self.network.borrow().connection_id(index).unwrap(),
                        self.network.borrow().token(index).unwrap(),
                        msg_string
                    )
                //self.node.borrow().sender.broadcast(msg_string)
            },
            _ => {
                self.node.borrow().sender.send("No node with that name")
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
        }
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
            // Construct the server
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