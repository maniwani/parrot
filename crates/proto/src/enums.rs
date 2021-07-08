
pub enum ConnectionState {
    Created,
    Connecting(usize, Instant),
    Authenticating(Instant),
	/// The connection has been authenticated and now packets containing
    /// actual data can be sent. If the connection is closed, the local peer
    /// will mark any unacknowledged as lost. 
    Connected,
    Disconnecting,
    Disconnected,
}

pub enum DisconnectReason {
    ConnectTokenExpired,
    ConnectTokenInvalid,
    ConnectionRequestTimeout,
    ConnectionAuthenticationTimeout,
    EncryptionInvalid,
    ProtocolVersionInvalid,
    ConnectionDenied,
    ConnectionAttemptsExhausted,
    ConnectionIdleTimeout,
    PeerConnectionIdleTimeout,
    Closed,
    PeerClosed,
    SendBufferIsFull,
    RecvBufferIsFull,
    PeerSendBufferIsFull,
    PeerRecvBufferIsFull,
    ExcessivePacketLoss,
    Unknown,
}

#[derive(Copy, Clone, Debug)]
pub enum Request {
    Connect,
    Disconnect,
    Accept,
    Deny,
}