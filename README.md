# Parrot

(WIP)

Parrot is a library of modular components for building cross-platform multiplayer games.

Parrot makes it possible—and as easy as possible—for you to write networked gameplay code that's nearly identical to non-networked gameplay code.

Simply pick and choose the features you want, and Parrot will do it all seamlessly behind the scenes.

## Features

- send
  - no internal IO (use whatever socket you want)
  - optionally-reliable messages
  - connections with multiple channels
- sync
  - your choice between authoritative and deterministic state replication
  - lightweight snapshots with lightning-fast (de)compression and (de)serialization
  - lag compensation
    - client-side prediction
    - server-side rewind
  - snapshot interpolation
    - smoothly corrects errors for rendering predicted entities
    - handles teleportation correctly
  - interest management (server can filter what it sends each client)
  - multiple players per client (splitscreen)
