# qbit

# ToDo
- [x] Add todo, lmao
- [x] Deserialize tracker's response into url
- [ ] kindly, explore ratatui
- [ ] don't give up, please
- [ ] make error for every module
- [ ] more tests!!!
- [ ] **`peer::session`:** Implement the main `run()` loop for message processing and lifecycle management.
- [ ] **`peer::session`:** Implement download logic in `peer_has_something_i_want` and `request_block`.
- [ ] **`peer::session`:** Implement upload logic to handle `Request` messages and unchoke peers.
- [ ] **`peer::session`:** Send actual peer messages when session state changes (e.g., send `Interested` message).
- [ ] **`peer::session`:** Complete the `handle_message` function for `Have`, `Request`, `Cancel`, and `Piece` messages.
- [ ] **`peer::message`:** Add a function to encode/serialize outgoing peer messages.
- [ ] **`peer::connection`:** Add a `write_message` function to send data to the peer.
- [ ] **`torrent` module:** Implement a bitfield or similar structure to track downloaded pieces.
- [ ] **Integration:** Instantiate and use `PeerSession` within the application.