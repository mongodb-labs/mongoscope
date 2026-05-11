mod templates;
pub use templates::all_templates;

use std::time::Duration;
use rand::{rngs::SmallRng, Rng, SeedableRng as _};
use tokio::sync::mpsc;

use crate::data::{
    model::QueryEntry,
    source::DataSource,
    types::*,
};
use templates::{build_filter, build_pipeline};

pub struct MockSource;

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

                let entry = QueryEntry {
                    id: QueryId::try_new(id).unwrap(),
                    t_ms: TimestampMs::new(t_ms),
                    latency_ms: LatencyMs::new(latency),
                    op: tpl.op.clone(),
                    coll: CollectionName::try_new(tpl.coll).unwrap(),
                    app: AppName::try_new(tpl.app).unwrap(),
                    plan: tpl.plan.clone(),
                    index: tpl.index.map(|i| IndexName::try_new(i).unwrap()),
                    docs_examined: tpl.docs_examined.map(DocsExamined::new),
                    docs_returned: tpl.docs_returned.map(DocsReturned::new),
                    filter: build_filter(tpl.filter_keys),
                    pipeline: build_pipeline(tpl.pipeline_stages),
                    update: None,
                    doc: None,
                    warn: tpl.warn.map(str::to_string),
                    slow: latency >= 1000 || tpl.slow,
                };

                if tx.send(entry).await.is_err() {
                    break;
                }

                id += 1;
                tokio::time::sleep(Duration::from_millis(80)).await;
            }
        });
    }
}
