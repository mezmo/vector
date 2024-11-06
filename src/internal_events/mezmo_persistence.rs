use crate::mezmo::persistence::RocksDBConnection;
use metrics::{counter, gauge};
use mezmo::MezmoContext;
use rocksdb::statistics::{Histogram, Ticker};
use vector_lib::internal_event::InternalEvent;

#[derive(Debug)]
pub struct MezmoPersistenceRocksDBTicker<'a> {
    pub ticker: Ticker,
    pub connection: &'a RocksDBConnection,
    pub mezmo_ctx: &'a MezmoContext,
}

/// Emit metrics for RocksDB Tickers
impl InternalEvent for MezmoPersistenceRocksDBTicker<'_> {
    fn emit(self) {
        counter!(
            self.ticker.to_string().replace('.', "_"),
            "account_id" => self.mezmo_ctx.account_id()
                .map(|uuid|  uuid.to_string())
                .unwrap_or("unknown".to_string())
        )
        .increment(self.connection.db_opts.get_ticker_count(self.ticker));
    }
}

pub struct MezmoPersistenceRocksDBHistogram<'a> {
    pub histogram: Histogram,
    pub connection: &'a RocksDBConnection,
    pub mezmo_ctx: &'a MezmoContext,
}

/// Emit metrics for RocksDB Histograms
/// RocksDB maintains an internal histogram for each statistic and does not provide
/// access to the currently-observed value. Instead, the metrics are converted to a gauge
/// with the current value of each aggregate component of the metric.
impl InternalEvent for MezmoPersistenceRocksDBHistogram<'_> {
    fn emit(self) {
        let account_id = self
            .mezmo_ctx
            .account_id()
            .map(|uuid| uuid.to_string())
            .unwrap_or("unknown".to_string());

        let name = self.histogram.to_string().replace('.', "_");
        let data = self.connection.db_opts.get_histogram_data(self.histogram);

        gauge!(format!("{name}_avg"), "account_id" => account_id.clone()).set(data.average());
        gauge!(format!("{name}_p95"),  "account_id" => account_id.clone()).set(data.p95());
        gauge!(format!("{name}_sum"),  "account_id" => account_id.clone()).set(data.sum() as f64);
        gauge!(format!("{name}_count"),  "account_id" => account_id).set(data.count() as f64);
    }
}
