use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Connection {
    pub local_addr: String,
    pub local_port: u16,
    pub remote_addr: String,
    pub remote_port: u16,
    pub protocol: Protocol,
    pub state: String,
    pub interface: String,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
    Other(String),
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
            Protocol::Other(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Process {
    pub name: String,
    pub pid: u32,
    pub path: Option<String>,
    pub connections: Vec<Connection>,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub rate_in: f64,
    pub rate_out: f64,
}

impl Process {
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortField {
    Name,
    Pid,
    Connections,
    BytesIn,
    BytesOut,
    RateIn,
    RateOut,
}

impl SortField {
    pub fn next(self) -> Self {
        match self {
            SortField::Name => SortField::Pid,
            SortField::Pid => SortField::Connections,
            SortField::Connections => SortField::BytesIn,
            SortField::BytesIn => SortField::BytesOut,
            SortField::BytesOut => SortField::RateIn,
            SortField::RateIn => SortField::RateOut,
            SortField::RateOut => SortField::Name,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SortField::Name => "Name",
            SortField::Pid => "PID",
            SortField::Connections => "Conn",
            SortField::BytesIn => "Down",
            SortField::BytesOut => "Up",
            SortField::RateIn => "Rate In",
            SortField::RateOut => "Rate Out",
        }
    }
}

#[derive(Debug, Default)]
pub struct NetworkSnapshot {
    pub processes: Vec<Process>,
    pub total_bytes_in: u64,
    pub total_bytes_out: u64,
    pub total_rate_in: f64,
    pub total_rate_out: f64,
    pub total_connections: usize,
}

impl NetworkSnapshot {
    pub fn from_processes(processes: Vec<Process>) -> Self {
        let total_bytes_in: u64 = processes.iter().map(|p| p.bytes_in).sum();
        let total_bytes_out: u64 = processes.iter().map(|p| p.bytes_out).sum();
        let total_rate_in: f64 = processes.iter().map(|p| p.rate_in).sum();
        let total_rate_out: f64 = processes.iter().map(|p| p.rate_out).sum();
        let total_connections: usize = processes.iter().map(|p| p.connection_count()).sum();
        NetworkSnapshot {
            processes,
            total_bytes_in,
            total_bytes_out,
            total_rate_in,
            total_rate_out,
            total_connections,
        }
    }
}

pub type DnsCache = HashMap<String, Option<String>>;
