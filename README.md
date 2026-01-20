# qbit-tui
A bittorrent client, implementated in Rust.

> [!WARNING]
> The api implementation is still in action. So don't expect to cargo run and get a favourable outcome.
> At the moment, the client only behaves as a free-rider, thus can't be taken in action.

> [!TIP] 
> Still in this state, some part of api can be seen in action.
> ```sh
> cargo run --bin <a binary>
> ```
> Replace `<a binary>` with binary with any one in `src/bin`.



# ToDo
- [x] Add todo, lmao
- [x] Deserialize tracker's response into url
- [ ] kindly, explore ratatui
- [ ] don't give up, please
- [-] `(new)` make error for every module
- [ ] `(new)` Handle pieces sent by peers
- [ ] `(new)` Limit active users, as configured
- [ ] more tests!!!
- [ ] Actually test your api, lmao
- [ ] Check if peer's sending/asking for a valid piece
- [x] Implement Debug for Messages, manually
- [x] Implement serializer for Message
- [x] prepare and send `Request Message`
- [x] and prolly test them too
- [ ] **`peer::session`:** Implement the main `run()` loop for message processing and lifecycle management.
- [ ] **`peer::session`:** Implement download logic in `peer_has_something_i_want` and `request_block`.
- [ ] **`peer::session`:** Implement upload logic to handle `Request` messages and unchoke peers.
- [ ] **`peer::session`:** Send actual peer messages when session state changes (e.g., send `Interested` message).
- [ ] **`peer::session`:** Complete the `handle_message` function for `Have`, `Request`, `Cancel`, and `Piece` messages.
- [ ] **`peer::message`:** Add a function to encode/serialize outgoing peer messages.
- [ ] **`peer::connection`:** Add a `write_message` function to send data to the peer.
- [ ] **`torrent` module:** Implement a bitfield or similar structure to track downloaded pieces.
- [ ] **Integration:** Instantiate and use `PeerSession` within the application.
