# rustysignal
[![Cargo](https://img.shields.io/crates/v/rustysignal.svg)](https://crates.io/crates/rustysignal) 

> :warning: The version on the master-branch is currently untested. If you want to use a more stable version, install it from cargo according to the instructions below.

A signaling server written in Rust for WebRTC that supports SSL and Push notifications.
The signaling server is used to enable nodes on the network to exchange metadata in order to establish a peer-to-peer connection.
This signaling server supplies the ability to set usernames on the network, and users have the ability to send messages to a specific peer, or broadcast messages to everyone on the network.

## Installation
*You need [**Cargo**](https://doc.rust-lang.org/cargo/getting-started/installation.html) to be able to install this binary.* 

Install the signaling server without SSL
(Suitable for local testing)

`cargo install rustysignal` 

If you want to enable SSL, make sure to include it as a feature
(Needed when using WebRTC in production)

`cargo install rustysignal --features ssl`

Once installed, you can start it by executing `rustysignal 127.0.0.1:3012` in your terminal, which will start the server and listen to messages on the address `127.0.0.1:3012`.

If you are using SSL, you will need to provide your certificate. 
`rustysignal 127.0.0.1:3015 <CERT> <KEY>`

### Push
If you want to use push, you will need to build rustysignal from source. Clone the master branch, and run the server with `--features push`. For both push and ssl functionality, run it with `--features 'ssl push'`
The push is sent by including a connection request in the request payload. I.e. `action: "connection_request`. See `src/push.rs` for more information.

## Connecting to the network as a peer
When connecting to the network, i.e. Websocket, one should provide a username as a simple argument.
> wss://signalserverhost?user=yourname

Peers can be found by:
  1. A one-to-one request via a provided username, which sends your information to only the node tied to that username.
  2. A one-to-all request, which sends your information to everyone on the network.
  3. A one-to-self request, which sends your information back to yourself.
  
To specify which type of method, specify it in your websocket send command in a field called `protocol`.

```
var json_message = { protocol: "one-to-one", to: "receiver_username", "action": actiontype, "data": data };
ws.send(JSON.stringify(json_message));
```
