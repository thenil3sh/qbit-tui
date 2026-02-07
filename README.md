# qbit-tui
A bittorrent client, implementated in Rust.

> [!WARNING]
> The api implementation is still in progress. So don't expect to `cargo run` and get a favourable outcome.
> At the moment, the client only behaves as a free-rider, thus shoudn't be taken in use.

> [!TIP] 
> Even in this state, some part of api can still be seen in action.
> ```sh
> cargo run --bin <a binary>
> ```
> Replace `<a binary>` with binary with any one in `src/bin`.



# ToDo
- [x] Add todo, lmao
- [x] Deserialize tracker's response into url
- [ ] kindly, explore ratatui
- [ ] don't give up, please
- [ ] `(new)` Support for multifile torrents
- [ ] make error for every module
- [x] save state to a file as well as retrive ⭐
- [x] Handle pieces sent by peers
- [ ] Limit active users, as configured
- [x] any new piece gets marked as bad :(
- [ ] more tests!!!
- [x] Remove hardcoded values
- [ ] Actually test your api, lmao
- [ ] Check if peer's sending/asking for a valid piece
- [x] Implement Debug for Messages, manually
- [x] Implement serializer for Message
- [x] prepare and send `Request Message`
- [x] and prolly test them too
