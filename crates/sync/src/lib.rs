mod config;
mod time;

pub use time::*;

pub struct PlayerId(u32);

pub struct EntityId {
    id: u64,
    input_source: Option<PlayerId>,
    state_source: Option<PlayerId>,
}