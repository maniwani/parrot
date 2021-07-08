use std::{collections::HashMap, net::UdpSocket, time::{Duration, Instant}, mem::MaybeUninit};

use std::{io, net::SocketAddr};

use super::{
    constants::*, 
    cursor::BytesMut,
    packet::{
        frames::{Frame, Header, PacketType},
        pool::{BufferHandle, BufferPool},
        sequence_buffer::{SequenceBuffer, SequenceNumber},
    },
};

type ConnectionId = u64;
type ChannelId = u64;

pub struct Connections {
    conn: HashMap<ConnectionId, Connection>,
    pool: BufferPool,
    config: Config,
}

impl Connections {
    pub fn recv_on(&mut self, socket: UdpSocket) -> io::Result<usize> {
        let handle = self.pool.acquire().unwrap();
        let buf = self.pool.get_mut(handle).unwrap();

        if let Ok((number_of_bytes, src_addr)) = socket.recv_from(buf) {
            let header = Header::read(buf).unwrap();
            let connection = self.connection.get_mut(&header.dst_id).unwrap();

            match header.packet_type {
                PacketType::Handshake => {
                    // handle request
                },
                PacketType::Data => {
                    while let Some(frame) = Frame::read(buf) {
                        match self {
                            Frame::Padding { len } => {
                                continue;
                            },
                            Frame::Ping => {
                                // queue ping to be sent back
                            },
                            Frame::Ack {
                                ack_sequence,
                                ack_mask,
                            } => {
                                connection.acknowledge(ack_sequence, ack_mask);
                            },
                            // TODO: Frame for creating channels.
                            Frame::Data {
                                channel_id,
                                channel_sequence,
                                fragment_index,
                                fragment_count,
                                len,
                            } => {
                                let channel = connection.channels.get_mut(&channel_id).unwrap();
                                // store incoming data
                            },
                        }
                    }
                }
            }
        } else {
            self.pool.release(handle);
        }

    }

    pub fn send_on(&mut self, socket: UdpSocket) -> io::Result<usize> {
        // messages from channels with the same guarantees can be packed together
        // iterate channels with same guarantees
        // iterate messages to be sent
        // if there's enough space in the packet, add frame
        // up to limit of number of packets
    }
}

pub struct SendPacket {
    pub(crate) sequence: u64,
    pub(crate) included: [Option<(ChannelId, SequenceNumber, u8)>; 8],
}

pub struct Connection {
    pub(crate) src_id: ConnectionId,
    pub(crate) dst_id: ConnectionId,
    pub(crate) peer_addr: SocketAddr,
    pub(crate) state: ConnectionState,
    pub(crate) acks: Acknowledgment,
    pub(crate) channels: HashMap<ChannelId, Channel>,
    pub(crate) send_buffer: SequenceBuffer<SendPacket>,
    pub(crate) time_created: Instant,
    pub(crate) time_latest_recv: Option<Instant>,
    pub(crate) time_latest_send: Option<Instant>,
    pub(crate) rtt: Duration,
    pub(crate) mtu: usize,
    // TODO: Add connection-level stats
}

impl Connection {
    /// The session index of the local endpoint.
    #[inline]
    pub fn src_id(&self) -> ConnectionId {
        self.src_id
    }

    /// The session index of the remote endpoint.
    #[inline]
    pub fn dst_id(&self) -> ConnectionId {
        self.dst_id
    }

    /// The current state of this connection.
    #[inline]
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// The [Instant] this connection was created.
    #[inline]
    pub fn time_created(&self) -> Instant {
        self.time_created
    }

    /// The [Instant] a packet was last received on this connection.
    #[inline]
    pub fn time_latest_recv(&self) -> Instant {
        self.time_latest_recv
    }

    /// The [Instant] a packet was last sent on this connection.
    #[inline]
    pub fn time_latest_send(&self) -> Instant {
        self.time_latest_send
    }

    /// The round-trip time of this connection as [Duration].
    #[inline]
    pub fn rtt(&self) -> Duration {
        self.rtt
    }

    /// The maximum size of packets on this connection (in bytes).
    #[inline]
    pub fn mtu(&self) -> usize {
        self.mtu
    }

    fn disconnect(&mut self, reason: DisconnectReason) {
        // send an event to invoke other stuff
        self.state = ConnectionState::Disconnecting;
    }

    pub(crate) fn update(&mut self, time: Instant) {
        // Check if connection token has expired.
        if time >= self.token_expire_time() {
            // send local event
            self.disconnect(DisconnectReason::ConnectionTokenExpired);
            return;
        }

        match self.state {
            // IP address can change while connecting and reconnecting
            ConnectionState::Connecting(ref mut attempts, ref mut last_attempt) => {
                // Have we exhausted all of our connection attempts?
                if *attempts >= self.config.max_connection_attempts() {
                    // send local event
                    self.disconnect(DisconnectReason::ConnectionAttemptsExhausted);
                    return;
                } 
                
                // No? Then is it time to resend request?
                if time.saturating_duration_since(*last_attempt) >= self.config.request_timeout() {                    
                    // send local event
                    // send connect request packet
                    *last_attempt = time;
                    *attempts += 1;
                    return;
                }
            },
            ConnectionState::Connected => {
                // Have we timed out?
                if time.saturating_duration_since(self.latest_recv) >= self.config.connection_timeout() {
                    // send local event
                    // send disconnect notification packet with timeout as reason
                    // flush send window
                    self.disconnect(DisconnectReason::ConnectionTimeout);
                    return;
                }

                // Do we have any packets to send?
                // No? Is it time to send another keep-alive packet?
            },
            ConnectionState::Disconnecting(reason) => {
                // send local event
                // send disconnect notification packet with reason
                self.state = ConnectionState::Disconnected;
            },
            ConnectionState::Disconnected(timeout) => {
                if time >= timeout {
                    // remove connection, increment generation
                }
            },
            _ => {},
        }
    }

    pub(crate) fn handle_request(&mut self, request: Request) {
        // ignore requests coming from disconnected connections
        if let Request::Disconnect(reason) = request {
            if self.state == ConnectionState::Connected {
                // flush send window (say packet lost)
                self.disconnect(reason);
                return;
            }
        }

        match (self.role, self.state, request) {
            (Role::Client, ConnectionState::Connecting, Request::Accepted) => {
                // send local event
                self.state = ConnectionState::Connected;
            },
            (Role::Client, ConnectionState::Connection, Request::Denied) => {
                self.disconnect(DisconnectReason::ConnectionDenied);
            },
            (Role::Server, ConnectionState::Created, Request::Connect) => {
                // send local event
                // TODO: Authentication
                self.state = ConnectionState::Connected;
                // send Request::Accepted
            },
            (Role::Server, ConnectionState::Connected, Request::Connect) => {
                // The requester has not received our acceptance.
                // send Request::Accepted
            },
            _ => {
                panic!("Invalid connection state: {}, {}, {}", self.role, self.state, request);
            }
        }
    }
}

pub enum Send {
    Unreliable,
    Reliable,
}

pub enum Receive {
    Unordered,
    Sequenced,
    Ordered,
}

pub struct SendInfo {
    addr: SocketAddr,
    time: Instant,
    delivered: bool,
}

pub struct RecvInfo {
    addr: SocketAddr,
    time: Instant,
    delivered: bool,
}

pub struct Acknowledgement {
    pub(crate) next_send: SequenceNumber,
    pub(crate) latest_recv: Option<SequenceNumber>,
    pub(crate) latest_recv_mask: u32,
    pub(crate) latest_send_acked: Option<SequenceNumber>,
    pub(crate) oldest_send_unacked: Option<SequenceNumber>,
    pub(crate) next_recv_ordered: Option<SequenceNumber>,
}

impl Acknowledgement {
    pub fn new() -> Self {
        // TODO: start at somewhat random values?
        Acknowledgement {
            next_send: 0,
            latest_recv: None,
            latest_recv_mask: 0,
            latest_send_acked: None,
            oldest_send_unacked: None,
            next_recv_ordered: None,
        }
    }
    
    /// The next packet to send to the remote endpoint of this channel.
    pub fn sequence(&self) -> SequenceNumber {
        self.next_send
    }
    
    /// Bit array of the last T::BITS packets received from the remote endpoint.
    pub fn latest_recv_mask(&self) -> u32 {
        self.latest_recv_mask
    }

    /// The last packet we received from the remote endpoint on this channel.
    pub fn latest_recv(&self) -> Option<SequenceNumber> {
        self.latest_recv
    }
    
    pub fn latest_send_acked(&self) -> Option<SequenceNumber> {
        self.latest_send_acked
    }

    pub fn next_recv_ordered(&self) -> Option<SequenceNumber> {
        self.next_recv_ordered
    }

    pub fn oldest_send_unacked(&self) -> Option<SequenceNumber> {
        self.oldest_send_unacked
    }
}

pub struct RecvMessage {
    pub(crate) sequence: u64,
    pub(crate) fragment_count: u8,
    pub(crate) fragment_recv: u8,
    pub(crate) fragment_data: [Option<(BufferHandle, usize, usize)>; MAX_FRAGMENTS],
    pub(crate) time_created: Instant,
    pub(crate) time_recv: Option<Instant>,
}

pub struct SendMessage {
    pub(crate) sequence: u64,
    pub(crate) fragment_count: u8,
    pub(crate) fragment_sent: u8,
    pub(crate) fragment_data: [Option<(BufferHandle, usize, usize)>; MAX_FRAGMENTS],
    pub(crate) fragment_status: [SendStatus; MAX_FRAGMENTS], 
    pub(crate) time_created: Instant,
    pub(crate) time_sent: Option<Instant>,
}

pub enum SendStatus {
    Unsent,
    Sent,
    Delivered,
    Lost,
}

pub struct Channel {
    pub(crate) id: u64,
    pub(crate) acks: Acknowledgement,
    pub(crate) send_guarantee: Send, 
    pub(crate) recv_guarantee: Receive,
    pub(crate) send_buffer: SequenceBuffer<SendMessage>,
    pub(crate) recv_buffer: SequenceBuffer<RecvMessage>,
    pub(crate) time_latest_send: Option<Instant>,
    pub(crate) time_latest_recv: Option<Instant>,
    // TODO: add statistics (# messages sent, received, etc.)
}

impl Channel {
    pub fn new(id: usize, send_guarantee: Send, recv_guarantee: Receive) -> Self {
        Self {
            id,
            send_guarantee,
            recv_guarantee,
            acks: Acknowledgement::default(),
            send_buffer: None,
            recv_buffer: None,
            time_latest_send: None,
            time_latest_recv: None,
        }
    }

    pub fn acknowledge(
        &mut self, 
        recv: SequenceNumber,
        acked: SequenceNumber,
        acked_mask: u64,
    ) {
        let gap;
        match self.acks.latest_recv {
            Some(latest_recv) => {
                if recv <= latest_recv {
                    // message is stale or duplicate
                    return;
                }
                
                gap = recv - latest_recv;
                if gap > self.recv_buffer.capacity() {
                    // disconnect
                    return;
                }
            }
            None => {
                gap = 0;
            }
        }

        self.acks.latest_send_acked = Some(acked);
        self.acks.latest_recv = Some(recv);
        self.acks.latest_recv_mask = {
            if gap >= REDUNDANT_ACK_MASK_BITS {
                1
            } else {
                (self.latest_recv_mask << gap) | 1   
            }
        };

        let start = self.acks.oldest_send_unacked.unwrap_or(0);
        let end = acked;

        for sequence in start..=end {
            if let Some(packet) = self.send_buffer.get(sequence) {
                if acked < sequence {
                    // All unacknowledged packets in flight are newer.
                    return;
                }
                
                let gap = acked - sequence;
                if (gap >= REDUNDANT_ACK_MASK_BITS as u64) || ((acked_mask & (1 << gap)) == 0) {
                    // Packet was *probably* lost.
                } else {
                    // Packet was delivered.
                }
    
                self.send_buffer.remove(sequence);
            }
        }       
    }
}

pub enum ErrorKind {
    FragmentIndexInvalid,
    FragmentIndexAlreadyReceived,
    FragmentCountInvalid,
    FragmentCountExceedsMax,
    MessageOlderThanThreshold,
    NotEnoughBuffersAvailable,
    SendMessageZeroLength,
}

pub struct ConnectionRef<'a> {
    connection: &'a mut Connection,
    channel: &'a mut Channel,
    pool: &'a mut BufferPool,
}

impl<'a> ConnectionRef<'a> {

    pub fn read(&mut self, handle: BufferHandle) {

        let now = Instant::now();

        // TODO: need to read all data frames
        let buf = {
            let slice = unsafe {
                MaybeUninit::slice_assume_init_mut(self.pool.get_mut(handle)?)
            };
            BytesMut::new(slice)
        };

        match Frame::read(&mut buf).unwrap() {
            Frame::Padding { len } => {
                todo!();
            },
            Frame::Ping => {
                todo!();
            },
            Frame::Ack {
                ack_sequence,
                ack_mask,
            } => {
                todo!();
            },
            Frame::Data {
                // TODO: channel_type,
                channel_id,
                channel_sequence,
                fragment_index,
                fragment_count,
                len,
            } => {
                self.connection.channels
                    .entry(&channel_id)
                    .or_insert(Channel::new(channel_id, send_guarantee, recv_guarantee))
                    .store_incoming_data(
                        channel_sequence,
                        fragment_index,
                        fragment_count,
                        handle,
                        buf.position(),
                        buf.position() + len as usize,
                        now,
                    );
            },
        }
    }

    // TODO: len is optional field (LSB in frame type 1 == has length, 0 == full length)
    pub fn store_incoming_data(
        &mut self,
        sequence: u64,
        fragment_index: u8,
        fragment_count: u8,
        handle: BufferHandle,
        start: usize,
        end: usize,
        instant: Instant,
    ) -> io::Result<()> {
        match self.channel.recv_guarantee {
            Receive::Unordered => {
                if let Some(latest_recv) = self.channel.acks.latest_recv {
                    if sequence < latest_recv.saturating_sub(self.channel.recv_buffer.capacity() as u64) {
                        return Err(ErrorKind::MessageOlderThanThreshold);
                    }
                }
            },
            Receive::Ordered => {
                if let Some(next_recv_ordered) = self.channel.acks.next_recv_ordered {
                    if sequence < next_recv_ordered {
                        return Err(ErrorKind::MessageOlderThanThreshold);
                    }
                }
            },
            Receive::Sequenced => {
                if let Some(latest_recv) = self.channel.acks.latest_recv {
                    if sequence < latest_recv {
                        return Err(ErrorKind::MessageOlderThanThreshold);
                    }
                }
            },
        }

        let message = {
            if let Some(Some(message)) = self.channel.recv_buffer.get_mut(sequence) {
                if fragment_count != message.fragment_count {
                    return Err(ErrorKind::FragmentCountInvalid);
                }
                if fragment_index >= message.fragment_count {
                    return Err(ErrorKind::FragmentIndexInvalid);
                }
                if message.fragment_data[fragment_index as usize].is_some() {
                    return Err(ErrorKind::FragmentIndexAlreadyReceived);
                }
                message
            }
            else {
                let index = self.channel.recv_buffer.index_of(sequence);
                if let (Some(sequence), Some(message)) = self.channel.recv_buffer.remove_index(index) {
                    // release buffers held by old message
                    for location in message.fragment_data.iter().flatten() {
                        self.pool.release(location.0);
                    }
                }
                self.channel.recv_buffer.insert(
                    sequence,
                    RecvMessage {
                        sequence,
                        fragment_count,
                        fragment_recv: 0,
                        fragment_data: [None; MAX_FRAGMENTS],
                        time_created: instant,
                        time_recv: None,
                    },
                )
            }
        };

        message.fragment_recv += 1;
        message.fragment_data[fragment_index as usize] = Some((handle, start, end));
       
        if message.fragment_recv == message.fragment_count {
            self.connection.time_latest_recv = Some(instant);
            self.channel.time_latest_recv = Some(instant);
            message.time_recv = Some(instant);

            let prev_recv = self.channel.acks.latest_recv.take();
            self.channel.acks.latest_recv = match prev_recv {
                None => Some(sequence),
                Some(latest_recv) => Some(latest_recv.max(sequence)),
            };
            
            match self.recv_guarantee {
                Receive::Unordered => (),
                Receive::Ordered => {
                    // return messages in the order they were sent
                    let start = self.channel.acks.next_recv_ordered.unwrap_or(0);
                    let end = self.channel.acks.latest_recv.unwrap_or(start);
                    for sequence in start..=end {
                        if let Some(Some(message)) = self.channel.recv_buffer.get(sequence) {
                            if message.fragment_recv == message.fragment_count {
                                // push event
                                self.channel.acks.next_recv_ordered = Some(sequence + 1);
                                continue;
                            }
                        }
                        self.channel.acks.next_recv_ordered = Some(sequence);
                        break;
                    }
                },
                Receive::Sequenced => {
                    let start = prev_recv.unwrap_or(0);
                    let end = self.channel.acks.latest_recv.unwrap_or(start);
                    for sequence in start..=end {
                        // complete messages that are not already delivered
                        todo!();
                    }
                },
            }
        }

        Ok(())
    }
    
    pub fn store_outgoing_data(&mut self, data: &[u8], instant: Instant) -> io::Result<()> {
        // TODO: Check for exceeded send window.
        if data.len() == 0 {
            return Err(ErrorKind::SendMessageZeroLength);
        }
        
        // calculate the number of fragments and check that it's valid
        let fragment_count = (data.len() / MAX_FRAGMENT_BYTES) + 
                                  ((data.len() % MAX_FRAGMENT_BYTES) != 0) as usize;
        if fragment_count > MAX_FRAGMENTS {
            return Err(ErrorKind::FragmentCountExceedsMax);
        }
        if fragment_count > self.pool.capacity_remaining() {
            return Err(ErrorKind::NotEnoughBuffersAvailable)
        }

        // TODO: add buffer for user data
        let sequence = self.channel.acks.next_send;
        self.channel.acks.next_send += 1;

        let message = self.channel.send_buffer
            .insert(
                sequence,
                SendMessage {
                    sequence,
                    fragment_count: u8::from(fragment_count),
                    fragment_sent: 0,
                    fragment_data: [None; MAX_FRAGMENTS],
                    fragment_status: [SendStatus::Unsent; MAX_FRAGMENTS],
                    time_created: instant,
                    time_sent: None,
                }
            );
        
        // write fragment frames
        for index in 0..fragment_count {
            let handle = self.pool.acquire()?;
            let buf = {
                let slice = unsafe {
                    MaybeUninit::slice_assume_init_mut(self.pool.get_mut(handle)?)
                };
                BytesMut::new(slice)
            };
            
            let start = index * MAX_FRAGMENT_BYTES;
            let end = (start + MAX_FRAGMENT_BYTES).min(data.len());
            let len = end - start;
            
            let header = Header::Short {
                dst_id: self.connection.dst_id,
                packet_type: PacketType::Data,
                packet_number: self.connection.acks.next_send,
            };
            
            let frame = Frame::Data {
                channel_id: self.channel.id,
                channel_sequence: sequence,
                fragment_count: u8::from(fragment_count),
                fragment_index: u8::from(index),
                len: u16::from(len),
            };
            
            // skip writing the header since we don't know what the packet sequence number is
            buf.advance(Header::short_header_bytes())?;
            frame.write(&mut buf)?;
            message.fragment_data[index] = Some((handle, buf.position(), len));
            buf.copy_from_slice(&data[start..end])?;
        }
        
        Ok(())
    }

    pub fn recv(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // split off into its own function
        // pop from event queue
        
        // stack allocation
        let scratch = [0u8; MAX_FRAGMENTS * MAX_FRAGMENT_BYTES];
        
        // channel -> recv buffer
        // next packet
        // write fragments into buf, pass up to caller
    }
    
    pub fn send(&mut self, socket: impl Socket) -> io::Result<()> {

        let instant = Instant::now();

        // reliable non-sequenced has head of line blocking (prioritize resending lost messages)
        // reliable sequenced is only reliable for the latest packet

        // array of (sequence, fragment) to go with packet sequence

        // send from unreliable channels, then from reliable channels
        // unreliable must send whole message

        use std::sync::mpsc::sync_channel;
        let (sender, receiver) = sync_channel::<SendMessage>(1024);
        sender.clone();
        
        match sender.try_send(t) {
            Ok(_) => {

            },
            Err(e) => {

            },
        }

        // for channel in unreliable channels with pending messages
        // basically send all of them, packed as much as possible

        // for channel in reliable channels with lost and pending messages
        // basically send all of them, packed as much as possible

        // If this exceeds upload bandwidth, can look into weighted queueing algorithms
        // (e.g. deficit round-robin) and static priorities (e.g. unreliable > reliable).

        match self.send_guarantee {
            Send::Unreliable => {
                let sequence = 0;
                let message = self.channel.send_buffer.get_mut(sequence).as_mut().unwrap();
                // send fragments
                // write header

                // send and release
            },
            Send::Reliable => {
                todo!();
                // start from the oldest message whose delivery hasn't been confirmed
                // iterate its fragments and write pending ones to a new payload
                // TODO: check if entire message can be sent
                // set time_sent once all fragments have been sent once
                // write header

                // send
            },
        }

        self.connection.time_latest_send = Some(instant);
        self.channel.time_latest_send = Some(instant);
        message.time_latest_send = Some(instant);

        Ok(())
    }
}