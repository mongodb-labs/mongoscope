#[derive(Debug, Clone)]
pub struct CollectionItem {
    pub name: String,
    pub requests: u32,
    pub active: bool,
}

impl CollectionItem {
    pub fn requests_str(&self) -> String {
        let r = self.requests;
        if r >= 1_000_000 {
            format!("{:.1}M reqs", r as f64 / 1_000_000.0)
        } else if r >= 1_000 {
            format!("{:.0}K reqs", r as f64 / 1_000.0)
        } else {
            format!("{} reqs", r)
        }
    }
}
