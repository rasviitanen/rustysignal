#[cfg(feature = "push")]
pub struct Node {
    pub owner: Option<String>,
    pub subscription: Option<String>,
    pub sender: ws::Sender
}

#[cfg(feature = "push")]
impl Node {
    pub fn new(sender: ws::Sender) -> Node {
        Node {
            owner: None,
            subscription: None,
            sender: sender
        }
    }
}

#[cfg(not(feature = "push"))]
pub struct Node {
    pub owner: Option<String>,
    pub sender: ws::Sender
}

#[cfg(not(feature = "push"))]
impl Node {
    pub fn new(sender: ws::Sender) -> Node {
        Node {
            owner: None,
            sender: sender
        }
    }
}