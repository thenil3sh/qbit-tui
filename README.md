# qbit-tui
A bittorrent client, implementated in Rust.

> [!NOTE] 
> **ARCHIVED**, yes\
> You see it, repo will stay archived for a while. I've got some more projects to work on! I'll be back soon, fr.

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
- [ ] `(next)` session and state migration to session::peer::bitfield
- [ ] `(next)` session runtime overhaul
- [ ] `(next)` more docs
- [ ] `(next)` local endpoint for ui :0
- [ ] kindly, explore ratatui
- [ ] don't give up, please
- [ ] make error for every module
- [x] save state to a file as well as retrive ⭐
- [x] Handle pieces sent by peers
- [ ] Limit active users, as configured
- [x] any new piece gets marked as bad :(
- [x] Remove hardcoded values
- [x] Implement Debug for Messages, manually
- [x] Implement serializer for Message
- [x] prepare and send `Request Message`
- [x] and prolly test them too
