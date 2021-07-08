type PacketNumber = u64;

pub struct Acknowledgement {
    pub(crate) next_packet_number: PacketNumber,
    pub(crate) last_delivered_packet_number: Option<PacketNumber>,
    pub(crate) last_recv_packet_number: Option<PacketNumber>,
    pub(crate) last_recv_packet_mask: u32,
}

impl Acknowledgement {
    pub fn new() -> Self {
        Acknowledgement {
            next_packet_number: 0, // make random?
            last_delivered_packet_number: u16::max_value(), // make random?
            last_recv_packet_number: u16::max_value(), // make random?
            last_recv_packet_mask: 0,
        }
    }
    
    /// The next packet to send to the remote endpoint of this channel.
    pub fn packet_number(&self) -> PacketNumber {
        self.next_packet_number
    }
    
    /// The last packet we received from the remote endpoint on this channel.
    pub fn ack_packet_number(&self) -> PacketNumber {
        self.last_recv_packet_number
    }
    
    /// Bitset of the last 32 packets received from the remote endpoint.
    pub fn ack_packet_mask(&self) -> u32 {
        self.last_recv_packet_mask
    }

    /// The number of sent packets newer than the one that was last acknowledged.
    pub fn sent_packets_in_flight(&self) -> usize {
        self.sent_packets.len()
    }

    pub fn packet_distance(p1: PacketNumber, p2: PacketNumber) -> Result<i16, Error> {
        // If you want to use non power-of-two bit lengths, you need to left shift
        // both arguments by WORD_LENGTH - packet_LENGTH bits, then subtract, then
        // right shift the result back down by the same amount.
        let distance = p1.wrapping_sub(p2) as i16;
        if distance == i16::min_value() {
            // The order of two numbers 2^(N-1) apart is undefined in signed arithmetic.
            return Err(());
        }
    
        Ok(distance)
    }
}