//! Metrics parsing and rendering.
//!
//! Handles parsing of promphp Redis format and rendering to Prometheus text format.

mod parser;
mod renderer;
mod types;

pub use parser::parse_promphp_metric;
pub use renderer::render_metrics;
pub use types::{Metric, MetricFamily, MetricType, Sample};
