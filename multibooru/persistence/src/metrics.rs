use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, HeaderValue},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use prometheus_client::{
    encoding::{text::encode, EncodeLabelSet, EncodeLabelValue},
    metrics::{counter::Counter, family::Family},
    registry::Registry,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue)]
pub enum RecordStatus {
    Ok,
    FailedToParse,
    FailedToSave,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct RecordLabels {
    pub status: RecordStatus,
}

pub struct Metrics {
    pub records: Family<RecordLabels, Counter>,
}

impl Metrics {
    pub fn record_ok(&mut self) {
        self.records
            .get_or_create(&RecordLabels {
                status: RecordStatus::Ok,
            })
            .inc();
    }
    pub fn record_failed_parse(&mut self) {
        self.records
            .get_or_create(&RecordLabels {
                status: RecordStatus::FailedToParse,
            })
            .inc();
    }
    pub fn record_failed_save(&mut self) {
        self.records
            .get_or_create(&RecordLabels {
                status: RecordStatus::FailedToSave,
            })
            .inc();
    }
}

pub async fn run_metrics_server(
    registry: Arc<Mutex<Registry>>,
    shutdown: tokio_util::sync::CancellationToken,
) {
    async fn serve_metrics(State(registry): State<Arc<Mutex<Registry>>>) -> impl IntoResponse {
        let reg = registry.lock().unwrap();
        let mut body = String::new();
        encode(&mut body, &reg).unwrap();
        let mut resp = Response::new(body);
        resp.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/openmetrics-text; version=1.0.0; charset=utf-8"),
        );
        resp
    }

    let app = Router::new()
        .route("/metrics", get(serve_metrics))
        .with_state(registry);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9100").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown.cancelled_owned())
        .await
        .unwrap();
}
