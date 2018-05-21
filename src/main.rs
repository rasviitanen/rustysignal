extern crate ws;
extern crate serde_json;

use std::str;
use std::rc::Rc;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;

use ws::{listen, Handler, Sender, Result, Message, Handshake, CloseCode};
use serde_json::Value;

struct Server {
    out: Sender,
    count: Rc<Cell<u32>>,
    connected_nodes: Rc<RefCell<HashMap<String, u32>>>,
}

impl Handler for Server {
    fn on_open(&mut self, handshake: Handshake) -> Result<()> {
        let arguments = handshake.request.resource()[2..].split("=");
        let argument_vector: Vec<&str> = arguments.collect();
        println!("{:?}", argument_vector);
        let username: &str = argument_vector[1];
        self.connected_nodes.borrow_mut().insert(username.to_string(), self.out.connection_id());
        Ok(self.count.set(self.count.get() + 1))
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let msg_string: &str = msg.as_text()?;
        let json_message: Value = serde_json::from_str(msg_string).unwrap();
        let send_to = json_message["to"].as_str().unwrap();
        self.out.send_to(*self.connected_nodes.borrow().get(send_to).unwrap(), msg_string)
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal =>
                println!("The client is done with the connection."),
            CloseCode::Away =>
                println!("The client is leaving the site."),
            CloseCode::Abnormal =>
                println!("Closing handshake failed! Unable to obtain closing status from client."),
            _ =>
                println!("The client encountered an error: {}", reason),
        }

        // The connection is going down, so we need to delete the node and decrement the count.
        self.connected_nodes.borrow_mut().remove("ALIAS HERE");
        self.count.set(self.count.get() - 1)
    }

    fn on_error(&mut self, err: ws::Error) {
        println!("The server encountered an error: {:?}", err);
    }
}

fn main() {
    // Cell gives us interior mutability
    let count = Rc::new(Cell::new(0));
    let connected_nodes = Rc::new(RefCell::new(HashMap::new()));
    listen("127.0.0.1:3012",
           |out| {
               Server { out, count: count.clone(), connected_nodes: connected_nodes.clone()}
           }
    ).unwrap()
} 