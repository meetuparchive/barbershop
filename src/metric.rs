use std::time::{SystemTime, UNIX_EPOCH};

/// Return datadog lambda counter metric
///
/// See the datadog [docs](https://docs.datadoghq.com/integrations/amazon_lambda/#lambda-metrics)
/// for more information
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

#[cfg(test)]
mod tests {
    use super::incr;

    #[test]
    fn without_tags() {
        let metric = incr("foo.bar", Vec::new()).expect("time is moving backwards");
        let parts = metric.split("|").collect::<Vec<_>>();
        assert_eq!(parts[4], "foo.bar");
    }

    #[test]
    fn with_tags() {
        let metric = incr("foo.bar", vec!["baz:boom".into(), "zoom".into()]).expect("time is moving backwards");
        let parts = metric.split("|").collect::<Vec<_>>();
        assert_eq!(parts[4], "foo.bar");
        assert_eq!(parts[5], "#baz:boom,zoom");
    }
}
