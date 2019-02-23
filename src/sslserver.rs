#[cfg(feature = "ssl")]
use std::str;
#[cfg(feature = "ssl")]
use std::rc::Rc;
#[cfg(feature = "ssl")]
use std::rc::Weak;
#[cfg(feature = "ssl")]
use std::cell::RefCell;
#[cfg(feature = "ssl")]
use std::collections::HashMap;
#[cfg(feature = "ssl")]
use std::thread::sleep;
#[cfg(feature = "ssl")]
use std::time::Duration;

#[cfg(feature = "ssl")]
use ws::{Handler, Sender, Result, Message, Handshake, CloseCode};
#[cfg(feature = "ssl")]
use serde_json::Value;

#[cfg(feature = "ssl")]
use std::fs::File;
#[cfg(feature = "ssl")]
use std::io::Read;

#[cfg(feature = "ssl")]
use openssl::pkey::PKey;
#[cfg(feature = "ssl")]
use openssl::ssl::{SslAcceptor, SslMethod, SslStream};
#[cfg(feature = "ssl")]
use openssl::x509::X509;

#[cfg(feature = "ssl")]
use ws::util::TcpStream;

#[cfg(feature = "ssl")]
#[derive(Default)]
struct Network {
    nodemap: Rc<RefCell<HashMap<String, Weak<RefCell<Node>>>>>,
}

#[cfg(feature = "ssl")]
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

#[cfg(feature = "ssl")]
struct Node {
    owner: Option<String>,
    sender: Sender,
}

#[cfg(feature = "ssl")]
struct Server {
    node: Rc<RefCell<Node>>,
    ssl: Rc<SslAcceptor>,
    network: Rc<RefCell<Network>>,
}

#[cfg(feature = "ssl")]
impl Handler for Server {
    fn on_open(&mut self, handshake: Handshake) -> Result<()> {
        println!("Channel open");
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

    fn upgrade_ssl_server(&mut self, sock: TcpStream) -> ws::Result<SslStream<TcpStream>> {
        println!("Server node upgraded");
        sleep(Duration::from_millis(200));
        self.ssl.accept(sock).map_err(From::from)
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

#[cfg(feature = "ssl")]
pub fn run() {
    let network = Rc::new(RefCell::new(Network::default()));
    // Setup logging
    env_logger::init();

    // setup command line arguments
    let matches = clap::App::new("Rustysignal")
        .version("2.0.0")
        .author("Rasmus Viitanen <rasviitanen@gmail.com>")
        .about("A secure signaling server implemented in Rust that can be used for e.g. WebRTC, see https://github.com/rasviitanen/rustysignal")
        .arg(
            clap::Arg::with_name("ADDR")
                .help("Address on which to bind the server.")
                .required(true)
                .index(1),
        )
        .arg(
            clap::Arg::with_name("CERT")
                .help("Path to the SSL certificate.")
                .required(true)
                .index(2),
        )
        .arg(
            clap::Arg::with_name("KEY")
                .help("Path to the SSL certificate key.")
                .required(true)
                .index(3),
        )
        .get_matches();

    let cert = {
        let data = read_file(matches.value_of("CERT").unwrap()).unwrap();
        X509::from_pem(data.as_ref()).unwrap()
    };

    let pkey = {
        let data = read_file(matches.value_of("KEY").unwrap()).unwrap();
        PKey::private_key_from_pem(data.as_ref()).unwrap()
    };

    let acceptor = Rc::new({
        println!("Building acceptor");
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder.set_private_key(&pkey).unwrap();
        builder.set_certificate(&cert).unwrap();

        builder.build()
    });

    println!("------------------------------------");
    println!("rustysignal is listening on securily on address");
    println!("wss://{}", matches.value_of("ADDR").unwrap());
    println!("To disable SSL you need to reinstall rustysignal using 'cargo install rustysignal --force");
    println!("-------------------------------------");

    ws::Builder::new()
        .with_settings(ws::Settings {
            encrypt_server: true,
            ..ws::Settings::default()
        })
        .build(|sender: ws::Sender| {
            println!("Building server");
            let node = Node { owner: None, sender };
            Server {
                node: Rc::new(RefCell::new(node)),
                ssl: acceptor.clone(),
                network: network.clone()
            }
        })
        .unwrap()
        .listen(matches.value_of("ADDR").unwrap())
    .unwrap();
} 

#[cfg(feature = "ssl")]
fn read_file(name: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(name)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}
