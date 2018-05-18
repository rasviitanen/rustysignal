#[macro_use]
extern crate lazy_static;
extern crate serde_json;
extern crate ws;
extern crate mio;

use std::rc::Rc;
use std::cell::Cell;
use mio::Token;

use std::collections::HashMap;
use std::cell::RefCell;

use ws::{listen, Handler, Sender, Result, Message, Handshake, CloseCode, Error};

struct Server {
    out: Sender,
    count: Rc<Cell<u32>>,
    nodes: Rc<RefCell<HashMap<String, Token>>>,
}

impl Handler for Server {
    fn on_open(&mut self, handshake: Handshake) -> Result<()> {
        let id = std::str::from_utf8(&handshake.request.headers().get(7).unwrap().1).unwrap();
        println!("{:?}", &handshake.request.headers());
        println!("{:?}", self.out.token());
        match handshake.peer_addr {
            Some(address) => self.nodes.borrow_mut().insert("rasmus".to_string(), self.out.token()),
            _ => None,
        };

        Ok(self.count.set(self.count.get() + 1))
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        // Tell the user the current count
        let msg_string: &str = msg.as_text()?;
        println!("{}", msg_string);
        println!("The number of live connections is {}", self.count.get());
        // Echo the message back

        self.out.broadcast(msg_string)
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away => println!("The client is leaving the site."),
            CloseCode::Abnormal => println!(
                "Closing handshake failed! Unable to obtain closing status from client."),
            _ => println!("The client encountered an error: {}", reason),
        }

        // The connection is going down, so we need to decrement the count
        self.count.set(self.count.get() - 1)
    }


//    fn on_error(&mut self, err: ws::Error) {
//        println!("The server encountered an error: {:?}", err);
//    }

}

fn main() {
    // Cell gives us interior mutability so we can increment
    // or decrement the count between handlers.
    // Rc is a reference-counted box for sharing the count between handlers
    // since each handler needs to own its contents.
    let count = Rc::new(Cell::new(0));
    let nodes = Rc::new(RefCell::new(HashMap::new()));
    listen("127.0.0.1:3012", |out| { Server { out, count: count.clone(), nodes: nodes.clone()} }).unwrap()
} 