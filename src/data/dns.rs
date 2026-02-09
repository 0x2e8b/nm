use std::net::IpAddr;
use tokio::sync::mpsc;

use super::model::DnsCache;

/// Performs reverse DNS lookups asynchronously and returns results via channel.
pub fn spawn_dns_resolver() -> (mpsc::Sender<String>, mpsc::Receiver<(String, Option<String>)>) {
    let (req_tx, mut req_rx) = mpsc::channel::<String>(256);
    let (res_tx, res_rx) = mpsc::channel::<(String, Option<String>)>(256);

    tokio::spawn(async move {
        while let Some(ip_str) = req_rx.recv().await {
            let res_tx = res_tx.clone();
            tokio::spawn(async move {
                let hostname = resolve_hostname(&ip_str);
                let _ = res_tx.send((ip_str, hostname)).await;
            });
        }
    });

    (req_tx, res_rx)
}

fn resolve_hostname(ip_str: &str) -> Option<String> {
    let ip: IpAddr = ip_str.parse().ok()?;
    // dns_lookup::lookup_addr does reverse DNS
    dns_lookup::lookup_addr(&ip).ok()
}

/// Update connection hostnames from the DNS cache and request lookups for unknown IPs.
pub fn update_dns(
    processes: &mut [super::model::Process],
    cache: &DnsCache,
    pending: &mut std::collections::HashSet<String>,
    req_tx: &mpsc::Sender<String>,
) {
    for proc in processes.iter_mut() {
        for conn in proc.connections.iter_mut() {
            let ip = &conn.remote_addr;
            if ip.is_empty() {
                continue;
            }
            if let Some(hostname) = cache.get(ip) {
                conn.hostname.clone_from(hostname);
            } else if !pending.contains(ip) {
                pending.insert(ip.clone());
                let _ = req_tx.try_send(ip.clone());
            }
        }
    }
}

/// Drain resolved DNS results into the cache.
pub fn drain_dns_results(
    rx: &mut mpsc::Receiver<(String, Option<String>)>,
    cache: &mut DnsCache,
    pending: &mut std::collections::HashSet<String>,
) {
    while let Ok((ip, hostname)) = rx.try_recv() {
        pending.remove(&ip);
        cache.insert(ip, hostname);
    }
}
