mod docs;
mod templates;
pub use docs::gen_response_docs;
pub use templates::all_templates;

use rand::{rngs::SmallRng, Rng, SeedableRng as _};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::data::{model::QueryEntry, source::DataSource, types::*};
use templates::{build_filter, build_pipeline};

pub struct MockSource;

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
        tokio::spawn(async move {
            let templates = all_templates();
            let mut rng = SmallRng::seed_from_u64(42);
            let mut id: u64 = 10_004;
            let mut t_ms: u64 = 0;

            loop {
                let tpl = &templates[id as usize % templates.len()];
                let jitter = 0.8 + rng.gen::<f64>() * 0.4;
                let latency = ((tpl.base_latency_ms as f64 * jitter) as u32).max(1);
                t_ms += 20 + rng.gen_range(0u64..180);

                let docs_returned = tpl.docs_returned;
                let response_docs =
                    gen_response_docs(tpl.coll, &tpl.op, docs_returned.unwrap_or(1) as usize);

                let rejected_plan_count: u8 = match &tpl.plan {
                    Some(crate::data::model::Plan::CollScan) => 1,
                    Some(crate::data::model::Plan::IxScan(_)) => 2,
                    Some(crate::data::model::Plan::IxScanLookup(_)) => 2,
                    Some(crate::data::model::Plan::IdHack) => 0,
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
                    plan: tpl.plan.clone(),
                    index: tpl.index.map(|i| IndexName::try_new(i).unwrap()),
                    docs_examined: tpl.docs_examined.map(DocsExamined::new),
                    docs_returned: docs_returned.map(DocsReturned::new),
                    filter: build_filter(tpl.filter_keys),
                    pipeline: build_pipeline(tpl.pipeline_stages),
                    update: None,
                    doc: None,
                    warn: tpl.warn.map(str::to_string),
                    slow: latency >= 1000 || tpl.slow,
                    conn_id,
                    lsid,
                    cluster_time,
                    response_docs,
                    rejected_plan_count,
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
