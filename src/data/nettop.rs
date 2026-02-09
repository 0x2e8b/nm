use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;

use super::model::{Connection, Process, Protocol};

/// Fetch a snapshot from nettop (without -P to get per-connection detail).
/// Uses `-x -J` for machine-readable CSV with selected columns.
pub async fn fetch_nettop_snapshot() -> Result<Vec<Process>, String> {
    let output = Command::new("nettop")
        .args(["-L", "1", "-x", "-J", "bytes_in,bytes_out"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .map_err(|e| format!("Failed to run nettop: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_nettop_output(&stdout)
}

/// Parse nettop CSV output into processes with connections.
///
/// Format (without -P):
/// ```text
/// ,bytes_in,bytes_out,
/// process_name.pid,bytes_in,bytes_out,
/// tcp4 192.168.0.1:12345<->1.2.3.4:443,bytes_in,bytes_out,
/// udp4 *:5353<->*:*,bytes_in,bytes_out,
/// next_process.pid,bytes_in,bytes_out,
/// ```
///
/// Process lines have name.pid format. Connection lines start with a
/// protocol prefix (tcp4, tcp6, udp4, udp6).
fn parse_nettop_output(output: &str) -> Result<Vec<Process>, String> {
    let mut processes: Vec<Process> = Vec::new();

    let lines: Vec<&str> = output.lines().collect();

    // Find header line
    let start = match lines.iter().position(|l| l.contains("bytes_in")) {
        Some(i) => i + 1,
        None => return Ok(Vec::new()),
    };

    let mut current_process: Option<Process> = None;

    for line in &lines[start..] {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let first_field = match line.split(',').next() {
            Some(f) => f.trim(),
            None => continue,
        };

        if is_connection_line(first_field) {
            // This is a connection line belonging to the current process
            if let Some(ref mut proc) = current_process {
                if let Some(conn) = parse_connection_line(line) {
                    proc.connections.push(conn);
                }
            }
        } else {
            // This is a process summary line — save previous and start new
            if let Some(proc) = current_process.take() {
                if proc.bytes_in > 0 || proc.bytes_out > 0 || !proc.connections.is_empty() {
                    processes.push(proc);
                }
            }
            current_process = parse_process_line(line);
        }
    }

    // Don't forget the last process
    if let Some(proc) = current_process {
        if proc.bytes_in > 0 || proc.bytes_out > 0 || !proc.connections.is_empty() {
            processes.push(proc);
        }
    }

    Ok(processes)
}

/// Check if a first CSV field is a connection line (starts with protocol prefix).
fn is_connection_line(first_field: &str) -> bool {
    first_field.starts_with("tcp4 ")
        || first_field.starts_with("tcp6 ")
        || first_field.starts_with("udp4 ")
        || first_field.starts_with("udp6 ")
}

/// Parse a process summary line: "ProcessName.PID,bytes_in,bytes_out,"
fn parse_process_line(line: &str) -> Option<Process> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.is_empty() {
        return None;
    }

    let id = parts[0].trim();
    let (name, pid) = split_name_pid(id);
    if name.is_empty() {
        return None;
    }

    let bytes_in = parts.get(1).and_then(|v| v.trim().parse::<u64>().ok()).unwrap_or(0);
    let bytes_out = parts.get(2).and_then(|v| v.trim().parse::<u64>().ok()).unwrap_or(0);

    Some(Process {
        name,
        pid,
        path: None,
        connections: Vec::new(),
        bytes_in,
        bytes_out,
        rate_in: 0.0,
        rate_out: 0.0,
    })
}

/// Parse a connection line: "tcp4 192.168.0.1:12345<->1.2.3.4:443,bytes_in,bytes_out,"
fn parse_connection_line(line: &str) -> Option<Connection> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.is_empty() {
        return None;
    }

    let desc = parts[0].trim();
    let bytes_in = parts.get(1).and_then(|v| v.trim().parse::<u64>().ok()).unwrap_or(0);
    let bytes_out = parts.get(2).and_then(|v| v.trim().parse::<u64>().ok()).unwrap_or(0);

    // desc = "tcp4 192.168.0.227:61859<->17.57.146.59:5223"
    // or    "udp6 *.5353<->*.*"
    let (proto_str, addr_part) = desc.split_once(' ')?;

    let protocol = match proto_str {
        "tcp4" | "tcp6" => Protocol::Tcp,
        "udp4" | "udp6" => Protocol::Udp,
        _ => Protocol::Other(proto_str.to_string()),
    };

    // Split on <-> separator
    let (local_str, remote_str) = addr_part.split_once("<->")?;

    let (local_addr, local_port) = parse_addr_port(local_str);
    let (remote_addr, remote_port) = parse_addr_port(remote_str);

    Some(Connection {
        local_addr,
        local_port,
        remote_addr,
        remote_port,
        protocol,
        state: String::new(),
        interface: String::new(),
        bytes_in,
        bytes_out,
        hostname: None,
    })
}

/// Split "ProcessName.12345" into ("ProcessName", 12345).
/// Handles names with dots like "com.apple.WebKit.1234".
fn split_name_pid(s: &str) -> (String, u32) {
    if let Some(last_dot) = s.rfind('.') {
        let potential_pid = &s[last_dot + 1..];
        if let Ok(pid) = potential_pid.parse::<u32>() {
            return (s[..last_dot].to_string(), pid);
        }
    }
    (s.to_string(), 0)
}

/// Parse address:port from nettop format.
/// IPv4: "192.168.1.1:443", "*:5353", "*:*"
/// IPv6: "fe80::1c9b:e73b:41dd:4aa1%en7.49152", "::1.8021", "*.*"
fn parse_addr_port(s: &str) -> (String, u16) {
    let s = s.trim();

    // Wildcard
    if s == "*:*" || s == "*.*" {
        return ("*".to_string(), 0);
    }

    // IPv4 style with colon separator: "192.168.0.1:443" or "*:5353"
    // Also try dot separator for IPv6: "fe80::1.49152", "::1.8021", "*.5353"
    // Strategy: try colon first; if port part doesn't parse, fall through to dot.
    if let Some(last_colon) = s.rfind(':') {
        let port_str = &s[last_colon + 1..];
        if let Ok(port) = port_str.parse::<u16>() {
            return (s[..last_colon].to_string(), port);
        }
    }

    // Dot separator (IPv6 nettop format or wildcard)
    if let Some(last_dot) = s.rfind('.') {
        let port_str = &s[last_dot + 1..];
        if let Ok(port) = port_str.parse::<u16>() {
            return (s[..last_dot].to_string(), port);
        }
    }

    (s.to_string(), 0)
}

/// Compute rates by comparing two snapshots taken `interval_secs` apart.
pub fn compute_rates(
    current: &mut [Process],
    previous: &HashMap<(String, u32), (u64, u64)>,
    interval_secs: f64,
) {
    for proc in current.iter_mut() {
        let key = (proc.name.clone(), proc.pid);
        if let Some(&(prev_in, prev_out)) = previous.get(&key) {
            let delta_in = proc.bytes_in.saturating_sub(prev_in);
            let delta_out = proc.bytes_out.saturating_sub(prev_out);
            proc.rate_in = delta_in as f64 / interval_secs;
            proc.rate_out = delta_out as f64 / interval_secs;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_name_pid() {
        let (name, pid) = split_name_pid("firefox.1234");
        assert_eq!(name, "firefox");
        assert_eq!(pid, 1234);
    }

    #[test]
    fn test_split_name_pid_with_dots() {
        let (name, pid) = split_name_pid("com.apple.WebKit.1234");
        assert_eq!(name, "com.apple.WebKit");
        assert_eq!(pid, 1234);
    }

    #[test]
    fn test_parse_addr_port_ipv4() {
        let (addr, port) = parse_addr_port("192.168.1.1:443");
        assert_eq!(addr, "192.168.1.1");
        assert_eq!(port, 443);
    }

    #[test]
    fn test_parse_addr_port_ipv6_dot() {
        let (addr, port) = parse_addr_port("::1.8021");
        assert_eq!(addr, "::1");
        assert_eq!(port, 8021);
    }

    #[test]
    fn test_parse_addr_port_wildcard() {
        let (addr, port) = parse_addr_port("*:*");
        assert_eq!(addr, "*");
        assert_eq!(port, 0);

        let (addr, port) = parse_addr_port("*.*");
        assert_eq!(addr, "*");
        assert_eq!(port, 0);
    }

    #[test]
    fn test_is_connection_line() {
        assert!(is_connection_line("tcp4 192.168.0.227:50448<->194.15.120.159:1194"));
        assert!(is_connection_line("udp6 *.5353<->*.*"));
        assert!(!is_connection_line("firefox.1234"));
        assert!(!is_connection_line("Microsoft Teams.1263"));
    }

    #[test]
    fn test_parse_full_output() {
        let output = r#",bytes_in,bytes_out,
apsd.376,7387,24329,
tcp4 192.168.0.227:61859<->17.57.146.59:5223,7387,24329,
mDNSResponder.417,1238931,266702,
udp6 *.5353<->*.*,542567,138705,
udp4 *:5353<->*:*,696930,128507,
OneDrive.857,11296,3375,
tcp4 192.168.0.227:50501<->172.211.123.248:443,11296,3375,
"#;
        let processes = parse_nettop_output(output).unwrap();
        assert_eq!(processes.len(), 3);

        let apsd = processes.iter().find(|p| p.name == "apsd").unwrap();
        assert_eq!(apsd.pid, 376);
        assert_eq!(apsd.bytes_in, 7387);
        assert_eq!(apsd.connections.len(), 1);
        assert_eq!(apsd.connections[0].remote_addr, "17.57.146.59");
        assert_eq!(apsd.connections[0].remote_port, 5223);

        let mdns = processes.iter().find(|p| p.name == "mDNSResponder").unwrap();
        assert_eq!(mdns.connections.len(), 2);
    }
}
