use std::{default::Default, time::Duration};

use super::constants::*;

#[derive(Clone, Debug)]
pub struct Config {
    /// The size of the underlying socket's internal buffer that holds incoming packets.
    socket_recv_buffer_bytes: usize,
    /// The size of the underlying socket's internal buffer that holds outgoing packets.
    socket_send_buffer_bytes: usize,
    /// The size of the event buffer into which we receive socket events.
    socket_event_buffer_size: usize,
    /// Make the underlying socket block if `true`, non-blocking otherwise.
    socket_should_block: bool,
    /// Polling for socket events blocks for this duration, in milliseconds.
    socket_polling_timeout: Option<Duration>,
    // -----
    /// The maximum number of fragments a payload can be split into.
    max_fragments: usize,
    /// The maximum size of a fragment.
    max_fragment_bytes: usize,
    /// The maximum size of a payload (before fragmentation).
    max_payload_bytes: usize,
    // -----
    /// The maximum number of connections. Guards against memory exhaustion.
    max_connections: usize,
    /// When no other packets are sent, a heartbeat will be sent with this interval.
    /// If `None`, no heartbeats will be sent.
    heartbeat_timeout: Option<Duration>,
    /// The amount of time that can pass without hearing from a peer before the connection is dropped.
    idle_timeout: Duration,
    /// The maximum chain of sent packets that can remain unacknowledged before the connection is dropped.
    max_packets_in_flight: usize,
    /// The factor which will smooth out network jitter (EWMA).
    rtt_smoothing_factor: f32,
    /// The maximum round trip time that can be considered healthy (in milliseconds).
    rtt_max_good_value: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            socket_recv_buffer_bytes: 256 * KiB,
            socket_send_buffer_bytes: 256 * KiB,
            socket_event_buffer_size: 1024,
            socket_should_block: false,
            socket_polling_timeout: Some(Duration::from_millis(0)),
            max_fragments: MAX_FRAGMENTS,
            max_fragment_bytes: MAX_FRAGMENT_BYTES,
            max_payload_bytes: MAX_FRAGMENTS * MAX_FRAGMENT_BYTES,
            max_connections: 32,
            heartbeat_timeout: None,
            idle_timeout: Duration::from_secs(5),
            max_packets_in_flight: 256,
            rtt_smoothing_factor: 0.1,
            rtt_max_good_value: Duration::from_millis(250),
        }
    }
}
