use crate::data::model::QueryEntry;

pub const BUCKET_COUNT: usize = 80;

#[derive(Debug, Clone, Default)]
pub struct BucketData {
    pub ok: u32,
    pub warn: u32,
    pub slow: u32,
}

/// Rolling 80-bucket histogram. Each bucket = latest N ms of data.
pub struct Buckets {
    pub data: [BucketData; BUCKET_COUNT],
    pub bucket_ms: u64,
    pub head: usize,
}

impl Buckets {
    pub fn new(bucket_ms: u64) -> Self {
        Self {
            data: std::array::from_fn(|_| BucketData::default()),
            bucket_ms,
            head: 0,
        }
    }

    pub fn push(&mut self, entry: &QueryEntry, now_ms: u64) {
        let bucket_idx = ((now_ms / self.bucket_ms) as usize) % BUCKET_COUNT;
        if bucket_idx != self.head {
            // Zero out newly entered bucket
            self.data[bucket_idx] = BucketData::default();
            self.head = bucket_idx;
        }
        let ms = entry.latency_ms.into_inner();
        let b = &mut self.data[bucket_idx];
        if ms >= 1000 { b.slow += 1; }
        else if ms >= 100 { b.warn += 1; }
        else { b.ok += 1; }
    }

    pub fn ordered(&self) -> impl Iterator<Item = &BucketData> {
        let start = (self.head + 1) % BUCKET_COUNT;
        (0..BUCKET_COUNT).map(move |i| &self.data[(start + i) % BUCKET_COUNT])
    }

    pub fn max_total(&self) -> u32 {
        self.data.iter().map(|b| b.ok + b.warn + b.slow).max().unwrap_or(1).max(1)
    }
}
