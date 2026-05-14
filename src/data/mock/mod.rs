mod docs;
mod templates;
pub use docs::gen_response_docs;
pub use templates::all_templates;

use rand::{rngs::SmallRng, Rng, SeedableRng as _};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::data::{
    model::{IndexSuggestion, Plan, QueryEntry, Suggestion},
    source::DataSource,
    types::*,
};
use templates::{build_filter, build_pipeline};

pub struct MockSource {
    pub applied_templates: Arc<Mutex<HashSet<usize>>>,
}

impl MockSource {
    pub fn new(applied_templates: Arc<Mutex<HashSet<usize>>>) -> Self {
        Self { applied_templates }
    }
}

fn hex_session_id(id: u64) -> String {
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        (id * 0x9e37_79b9_7f4a_7c15) & 0xffff_ffff,
        (id >> 16) & 0xffff,
        0x4000 | ((id >> 32) & 0x0fff),
        0x8000 | ((id >> 48) & 0x3fff),
        id & 0xffff_ffff_ffff,
    )
}

impl DataSource for MockSource {
    fn start(self: Box<Self>, tx: mpsc::Sender<QueryEntry>) {
        let applied_templates = self.applied_templates.clone();
        tokio::spawn(async move {
            let templates = all_templates();
            let mut rng = SmallRng::seed_from_u64(42);
            let mut id: u64 = 10_004;
            let mut t_ms: u64 = 0;

            loop {
                let tpl_idx = id as usize % templates.len();
                let tpl = &templates[tpl_idx];
                let jitter = 0.8 + rng.gen::<f64>() * 0.4;
                let base_latency = ((tpl.base_latency_ms as f64 * jitter) as u32).max(1);
                t_ms += 20 + rng.gen_range(0u64..180);

                let is_collscan = matches!(tpl.plan, Some(Plan::CollScan));
                let examined = tpl.docs_examined.unwrap_or(1) as f32;
                let returned = tpl.docs_returned.unwrap_or(1) as f32;
                let selectivity = (returned / examined).clamp(0.001, 1.0);

                let index_applied = is_collscan
                    && applied_templates
                        .lock()
                        .map(|s| s.contains(&tpl_idx))
                        .unwrap_or(false);

                let (latency, plan, docs_examined, warn, slow, suggestions) = if index_applied {
                    let after_total = (base_latency as f32 * selectivity).max(2.0) as u32;
                    let latency = ((after_total as f64 * jitter) as u32).max(1);
                    (
                        latency,
                        Some(Plan::IxScan(IndexName::try_new("suggested_idx_1").unwrap())),
                        tpl.docs_returned.map(DocsExamined::new),
                        None,
                        latency >= 1000,
                        vec![],
                    )
                } else if is_collscan {
                    let after_total = (base_latency as f32 * selectivity).max(2.0);
                    let suggestions = vec![Suggestion::CreateIndex(IndexSuggestion {
                        ixscan_ms: (after_total * 0.30) as u32,
                        fetch_ms: (after_total * 0.55) as u32,
                        sort_ms: (after_total * 0.12) as u32,
                        limit_ms: 1,
                    })];
                    (
                        base_latency,
                        tpl.plan.clone(),
                        tpl.docs_examined.map(DocsExamined::new),
                        tpl.warn.map(str::to_string),
                        base_latency >= 1000 || tpl.slow,
                        suggestions,
                    )
                } else {
                    (
                        base_latency,
                        tpl.plan.clone(),
                        tpl.docs_examined.map(DocsExamined::new),
                        tpl.warn.map(str::to_string),
                        base_latency >= 1000 || tpl.slow,
                        vec![],
                    )
                };

                let docs_returned = tpl.docs_returned;
                let response_docs =
                    gen_response_docs(tpl.coll, &tpl.op, docs_returned.unwrap_or(1) as usize);

                let rejected_plan_count: u8 = match &plan {
                    Some(Plan::CollScan) => 1,
                    Some(Plan::IxScan(_)) => 2,
                    Some(Plan::IxScanLookup(_)) => 2,
                    Some(Plan::IdHack) => 0,
                    _ => 0,
                };

                let conn_id = ConnId::new(10001 + (id % 20) as u32);

                let lsid = if id % 10 >= 3 {
                    Some(format!("UUID(\"{}\")", hex_session_id(id)))
                } else {
                    None
                };

                let cluster_time =
                    Some(format!("Timestamp({}, 1)", 1_700_000_000u64 + t_ms / 1000));

                let entry = QueryEntry {
                    id: QueryId::try_new(id).unwrap(),
                    t_ms: TimestampMs::new(t_ms),
                    latency_ms: LatencyMs::new(latency),
                    op: tpl.op.clone(),
                    db: DatabaseName::try_new(tpl.db).unwrap(),
                    coll: CollectionName::try_new(tpl.coll).unwrap(),
                    app: AppName::try_new(tpl.app).unwrap(),
                    plan,
                    index: tpl.index.map(|i| IndexName::try_new(i).unwrap()),
                    docs_examined,
                    docs_returned: docs_returned.map(DocsReturned::new),
                    filter: build_filter(tpl.filter_keys),
                    pipeline: build_pipeline(tpl.pipeline_stages),
                    update: None,
                    doc: None,
                    warn,
                    slow,
                    conn_id,
                    lsid,
                    cluster_time,
                    response_docs,
                    rejected_plan_count,
                    suggestions,
                };

                if tx.send(entry).await.is_err() {
                    break;
                }

                id += 1;
                tokio::time::sleep(Duration::from_millis(350)).await;
            }
        });
    }
}
