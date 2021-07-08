pub const STANDARD_HEADER_BYTES: usize = 5;
pub const FRAGMENT_FRAME_BYTES: usize = 4;
pub const ACK_FRAME_BYTES: usize = 8;
pub const ARRANGING_HEADER_BYTES: usize = 3;
pub const IPV6_HEADER_BYTES: usize = 40;
pub const UDP_HEADER_BYTES: usize = 8;
pub const MAX_PACKET_BYTES: usize = 1280; // min. 1280, max. 1500
pub const MAX_PAYLOAD_BYTES: usize = MAX_PACKET_BYTES - IPV6_HEADER_BYTES - UDP_HEADER_BYTES; // min. 1232, max. 1452
pub const MAX_FRAGMENTS: usize = 256;
pub const MAX_FRAGMENT_BYTES: usize = MAX_PAYLOAD_BYTES - FRAGMENT_FRAME_BYTES;
pub const MAX_MESSAGE_BYTES: usize = MAX_FRAGMENTS * MAX_FRAGMENT_BYTES;
pub const DEFAULT_RTT_MS: usize = 100;
pub const DEFAULT_CHANNEL_ID: usize = 0;
pub const PROTOCOL_VERSION: &str = "parrot-0.0.1";

pub(crate) const REDUNDANT_ACK_MASK_BITS: usize = 64;
pub(crate) const DEFAULT_SEND_WINDOW_SIZE: usize = 256;
