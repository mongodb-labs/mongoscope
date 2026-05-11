#[derive(Debug, Clone)]
pub struct CollectionItem {
    pub name: String,
    pub docs: u64,
    pub size: String,
    pub idx: u8,
    pub active: bool,
}

impl CollectionItem {
    pub fn docs_str(&self) -> String {
        let d = self.docs;
        if d >= 1_000_000 {
            format!("{:.1}M docs", d as f64 / 1_000_000.0)
        } else if d >= 1_000 {
            format!("{:.0}K docs", d as f64 / 1_000.0)
        } else {
            format!("{} docs", d)
        }
    }
}
