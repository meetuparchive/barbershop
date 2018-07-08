// Std
use std::time::{SystemTime, UNIX_EPOCH};

pub fn incr(metric_name: &str, tags: Vec<String>) -> Option<String> {
  SystemTime::now().duration_since(UNIX_EPOCH).ok().map(|d| {
    format!(
      "MONITORING|{timestamp}|{value}|count|{metric_name}|#{tags}",
      timestamp = d.as_secs(),
      value = 1,
      metric_name = metric_name,
      tags = tags.join(",")
    )
  })
}
