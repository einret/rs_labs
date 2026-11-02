use futures::stream::{self, StreamExt};

use crate::client::error::RekorApiError;
use crate::client::http::RekorApi;
use crate::client::result::{LeakResult, LeakStore};

struct UnitOutcome {
    unit_start: u64,
    result: Result<Vec<LeakResult>, RekorApiError>,
}

pub(crate) async fn scan(
    client: &dyn RekorApi,
    start: u64,
    count: u64,
    concurrency: usize,
) -> LeakStore {
    let mut store = LeakStore::default();

    let end = start.saturating_add(count);
    let width = client.scan_unit_width();
    let unit_starts = (start / width)..=((end - 1) / width);

    let mut stream = stream::iter(unit_starts)
        .map(|unit| unit * width)
        .map(|unit_start| async move {
            UnitOutcome {
                unit_start,
                result: client.get_tile_containing(unit_start).await,
            }
        })
        .buffer_unordered(concurrency);

    while let Some(outcome) = stream.next().await {
        match outcome {
            UnitOutcome {
                result: Ok(leaks), ..
            } => {
                for leak in leaks {
                    if leak.log_index >= start && leak.log_index < end {
                        store.insert(leak);
                    }
                }
            }
            UnitOutcome {
                unit_start,
                result: Err(e),
            } => {
                eprintln!("[orchestrator] log index {unit_start}: {e}")
            }
        }
    }

    store
}
