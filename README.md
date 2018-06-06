# rustysignal
A signaling server written in Rust for WebRTC.
The server is used to enable nodes on the network to exchange metadata in order to establish a peer-to-peer connection.

## Connecting to the network
When connecting to the network, i.e. Websocket, one should provide a username as a simple argument.
> ws://signalserverhost?user=yourname

Peers can be found by:
  1. A one-to-one request via a provided username, which sends your information to only the node tied to that username.
  2. A one-to-all request, which sends your information to everyone on the network.
  3. A one-to-self request, which sends your information back to yourself.
  
To specify which type of method, specify it in your websocket send command in a field called `protocol`.

```
var json_message = {protocol: "one-to-one", to: "you", "action": actiontype, "data": data};
ws.send(JSON.stringify(json_message));
```
