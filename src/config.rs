use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "nm", about = "Network Monitor TUI — lightweight terminal network traffic viewer")]
pub struct Config {
    /// Refresh interval in seconds
    #[arg(short, long, default_value_t = 2)]
    pub interval: u64,

    /// Initial sort field: name, pid, conn, down, up, rate-in, rate-out
    #[arg(short, long, default_value = "rate-in")]
    pub sort_by: String,
}

impl Config {
    pub fn parse_sort_field(&self) -> crate::data::model::SortField {
        match self.sort_by.as_str() {
            "name" => crate::data::model::SortField::Name,
            "pid" => crate::data::model::SortField::Pid,
            "conn" => crate::data::model::SortField::Connections,
            "down" => crate::data::model::SortField::BytesIn,
            "up" => crate::data::model::SortField::BytesOut,
            "rate-in" => crate::data::model::SortField::RateIn,
            "rate-out" => crate::data::model::SortField::RateOut,
            _ => crate::data::model::SortField::RateIn,
        }
    }
}
