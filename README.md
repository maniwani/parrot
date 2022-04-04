# Parrot

(WIP)

Parrot is a library of modular components for building cross-platform multiplayer games.

Parrot makes it possible—and as easy as possible—for you to write networked gameplay code that's nearly identical to non-networked gameplay code.

Simply pick and choose the features you want, and Parrot will do it all seamlessly behind the scenes.

## (Planned) Features

In no particular order...

sending

- no internal IO (use whatever socket you want)
- connections
  - connect / disconnect events
  - multiple channels (send unreliable or reliable, receive unordered, ordered (reliable-only), or sequenced)
  - heartbeats and idle timeout
- somewhere to hook custom authentication
- metrics

syncing

- user choice between authoritative and deterministic state replication
- lightweight snapshots with lightning-fast (de)compression and (de)serialization
- multiple players per connection (splitscreen, bots)
- lag compensation
  - client-side prediction
  - server-side rewind
- snapshot interpolation
  - smoothly correct errors for rendering predicted entities
  - correctly handle teleportation
- interest management (server can filter what it sends each client)
  - scope and prioritization
    - age-based
    - proximity-based
    - other (field-of-vision, gameplay rules, etc.)
- "RPCs" (one-off inputs and requests, e.g. "respawn me," "choose this loadout", etc.)
- metrics

integration

- bevy

## License

Parrot is dual-licensed under either of [Apache License, Version
2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual-licensed as above, without any additional terms or conditions.
