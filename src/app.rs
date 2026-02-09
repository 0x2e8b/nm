use std::collections::{HashMap, HashSet, VecDeque};

use tokio::sync::mpsc;

use crate::data::dns;
use crate::data::model::{DnsCache, NetworkSnapshot, Process, SortField};
use crate::data::nettop;
use crate::data::procinfo;

const BANDWIDTH_HISTORY_LEN: usize = 60;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActiveTab {
    Processes,
    Connections,
    Overview,
}

impl ActiveTab {
    pub fn next(self) -> Self {
        match self {
            ActiveTab::Processes => ActiveTab::Connections,
            ActiveTab::Connections => ActiveTab::Overview,
            ActiveTab::Overview => ActiveTab::Processes,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ActiveTab::Processes => ActiveTab::Overview,
            ActiveTab::Connections => ActiveTab::Processes,
            ActiveTab::Overview => ActiveTab::Connections,
        }
    }
}

pub struct App {
    pub active_tab: ActiveTab,
    pub snapshot: NetworkSnapshot,
    pub process_index: usize,
    pub connection_index: usize,
    pub sort_field: SortField,
    pub filter_text: Option<String>,
    pub filter_input: String,
    pub filtering: bool,
    pub show_help: bool,
    pub paused: bool,
    pub should_quit: bool,
    pub bandwidth_history: VecDeque<f64>,

    // Internal state for rate computation
    prev_bytes: HashMap<(String, u32), (u64, u64)>,

    // DNS
    dns_cache: DnsCache,
    dns_pending: HashSet<String>,
    dns_req_tx: mpsc::Sender<String>,
    dns_res_rx: mpsc::Receiver<(String, Option<String>)>,

    // Config
    pub interval_secs: u64,
}

impl App {
    pub fn new(sort_field: SortField, interval_secs: u64) -> Self {
        let (dns_req_tx, dns_res_rx) = dns::spawn_dns_resolver();
        App {
            active_tab: ActiveTab::Processes,
            snapshot: NetworkSnapshot::default(),
            process_index: 0,
            connection_index: 0,
            sort_field,
            filter_text: None,
            filter_input: String::new(),
            filtering: false,
            show_help: false,
            paused: false,
            should_quit: false,
            bandwidth_history: VecDeque::with_capacity(BANDWIDTH_HISTORY_LEN),
            prev_bytes: HashMap::new(),
            dns_cache: HashMap::new(),
            dns_pending: HashSet::new(),
            dns_req_tx,
            dns_res_rx,
            interval_secs,
        }
    }

    pub async fn update_data(&mut self) {
        if self.paused {
            return;
        }

        // Drain any DNS results
        dns::drain_dns_results(&mut self.dns_res_rx, &mut self.dns_cache, &mut self.dns_pending);

        // Fetch nettop data
        let mut processes = match nettop::fetch_nettop_snapshot().await {
            Ok(p) => p,
            Err(_) => return,
        };

        // Compute rates
        let interval = self.interval_secs as f64;
        nettop::compute_rates(&mut processes, &self.prev_bytes, interval);

        // Save current bytes for next rate computation
        self.prev_bytes = processes
            .iter()
            .map(|p| ((p.name.clone(), p.pid), (p.bytes_in, p.bytes_out)))
            .collect();

        // Enrich with process paths
        procinfo::enrich_process_paths(&mut processes);

        // Update DNS
        dns::update_dns(
            &mut processes,
            &self.dns_cache,
            &mut self.dns_pending,
            &self.dns_req_tx,
        );

        // Sort
        self.sort_processes(&mut processes);

        // Build snapshot
        self.snapshot = NetworkSnapshot::from_processes(processes);

        // Update bandwidth history
        let total_rate = self.snapshot.total_rate_in + self.snapshot.total_rate_out;
        if self.bandwidth_history.len() >= BANDWIDTH_HISTORY_LEN {
            self.bandwidth_history.pop_front();
        }
        self.bandwidth_history.push_back(total_rate);

        // Clamp indices
        let max_proc = self.snapshot.processes.len().saturating_sub(1);
        if self.process_index > max_proc {
            self.process_index = max_proc;
        }
    }

    fn sort_processes(&self, processes: &mut Vec<Process>) {
        match self.sort_field {
            SortField::Name => processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
            SortField::Pid => processes.sort_by_key(|p| p.pid),
            SortField::Connections => {
                processes.sort_by(|a, b| b.connection_count().cmp(&a.connection_count()))
            }
            SortField::BytesIn => {
                processes.sort_by(|a, b| b.bytes_in.cmp(&a.bytes_in))
            }
            SortField::BytesOut => {
                processes.sort_by(|a, b| b.bytes_out.cmp(&a.bytes_out))
            }
            SortField::RateIn => {
                processes.sort_by(|a, b| b.rate_in.partial_cmp(&a.rate_in).unwrap_or(std::cmp::Ordering::Equal))
            }
            SortField::RateOut => {
                processes.sort_by(|a, b| b.rate_out.partial_cmp(&a.rate_out).unwrap_or(std::cmp::Ordering::Equal))
            }
        }
    }

    pub fn filtered_processes(&self) -> Vec<&Process> {
        let filter = match &self.filter_text {
            Some(f) if !f.is_empty() => Some(f.to_lowercase()),
            _ => None,
        };

        self.snapshot
            .processes
            .iter()
            .filter(|p| {
                if let Some(ref f) = filter {
                    p.name.to_lowercase().contains(f)
                        || p.path.as_deref().unwrap_or("").to_lowercase().contains(f)
                        || p.pid.to_string().contains(f)
                } else {
                    true
                }
            })
            .collect()
    }

    pub fn nav_up(&mut self) {
        match self.active_tab {
            ActiveTab::Processes => {
                self.process_index = self.process_index.saturating_sub(1);
            }
            ActiveTab::Connections => {
                self.connection_index = self.connection_index.saturating_sub(1);
            }
            _ => {}
        }
    }

    pub fn nav_down(&mut self) {
        match self.active_tab {
            ActiveTab::Processes => {
                let max = self.filtered_processes().len().saturating_sub(1);
                if self.process_index < max {
                    self.process_index += 1;
                }
            }
            ActiveTab::Connections => {
                self.connection_index += 1;
            }
            _ => {}
        }
    }

    pub fn cycle_sort(&mut self) {
        self.sort_field = self.sort_field.next();
    }

    pub fn enter_filter(&mut self) {
        self.filtering = true;
        self.filter_input.clear();
    }

    pub fn apply_filter(&mut self) {
        self.filtering = false;
        if self.filter_input.is_empty() {
            self.filter_text = None;
        } else {
            self.filter_text = Some(self.filter_input.clone());
        }
    }

    pub fn cancel_filter(&mut self) {
        self.filtering = false;
        self.filter_text = None;
        self.filter_input.clear();
    }

    pub fn drill_down(&mut self) {
        if self.active_tab == ActiveTab::Processes {
            // Get the selected process name before mutating
            let name = self
                .filtered_processes()
                .get(self.process_index)
                .map(|p| p.name.clone());

            self.active_tab = ActiveTab::Connections;
            if let Some(name) = name {
                self.filter_text = Some(name.clone());
                self.filter_input = name;
            }
            self.connection_index = 0;
        }
    }
}
