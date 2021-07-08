/// Controls the capabilities of the application.
pub enum AppMode {
    /// Playable, can connect to a server or relay, not (fully) authoritative.
    Client,
    /// Authoritative and open to client connections, but not playable.
    /// 
    /// Also known as dedicated server.
    Server,
    /// Playable, authoritative, and open to client connections.
    /// 
    /// Also known as listen server.
    Host,
    /// Playable, deterministic, and can connect to a server or relay.
    Peer,
    /// Open to client connections, but not playable or authortitative. 
    Relay,
    /// Playable and authoritative, but no connection capability.
    Offline,
}

/// Controls how far clients can simulate ahead of confirmed game state to reduce input latency.
pub enum Prediction {
    /// Clients do not predict. Also known as lockstep.
    None,
    /// Clients predict up to a certain number of simulation steps.
    /// If their round-trip time exceeds that amount, the remainder is covered by input delay.
    /// 
    /// Primarily intended for deterministic apps.
    Bounded,
    /// Clients predict as many simulation steps as necessary to cover their full round-trip time.
    /// 
    /// Primarily intended for authoritative apps.
    Unbounded,
}

/// Controls which mechanism keeps everyone in sync.
pub enum Replication {
    /// Clients receive confirmed state updates from a deterministic simulation.
    Deterministic,
    /// Clients receive confirmed state updates from a server.
    Authoritative,
}

/// Controls who owns the networked state, i.e. who has write permission.
/// 
/// Only for apps with [`Authoritative`](Replication::Authoritative) replication.
pub enum Authority {
    /// The server owns everything.
    Server,
    /// Ownership is distributed between clients.
    Client,
    /// Ownership is distributed between clients and the server. 
    Distributed,
}

/// Controls what information is included in state updates.
/// 
/// Only for apps with [`Authoritative`](Replication::Authoritative) replication.
pub enum Updates {
    /// Each update sent to a client contains the entire networked state.
    Full,
    /// Each update sent to a client contains a subset of the networked state. 
    Filtered,
}

enum Unit {
    Time(std::time::Duration),
    Ticks(usize),
}

/// Controls how the server compensates lag between clients, i.e. for hit detection.
pub enum Rewind {
    /// The server does not rewind state.
    None,
    /// The server rewinds state to the nearest tick.
    NearestTick,
    /// The server rewinds state to the exact interpolated instants clients see.
    Exact,
}

pub struct LagCompensation {
    local: Prediction,
    remote: Rewind,
    min_input_delay: Unit,
    max_ping: Unit,
    packet_loss_threshold: f64,
    client_tick_send_ratio: usize,
    server_tick_send_ratio: usize,
}

pub struct Simulation {
    tick_rate: usize,
    max_entities: usize,
    max_players: usize,
    max_ping: Unit,
}