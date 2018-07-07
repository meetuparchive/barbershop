#[macro_use]
extern crate cpython;
extern crate crypto;
#[macro_use]
extern crate lando;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate envy;
extern crate hex;

// Std
use std::time::{SystemTime, UNIX_EPOCH};

// Third party
use crypto::hmac::Hmac;
use crypto::mac::{Mac, MacResult};
use crypto::sha1::Sha1;
use hex::FromHex;
use lando::RequestExt;

mod github;

#[derive(Deserialize)]
struct Config {
    github_token: String,
    github_webhook_secret: String,
}

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

/// Return true if webhook was authenticated, false otherwise
fn authenticated(request: &lando::Request, secret: &String) -> bool {
    request
        .headers()
        .get("X-Hub-Signature")
        .and_then(|value| {
            // strip off `sha1=` and get hex bytes
            Vec::from_hex(value.to_str().expect("invalid header")[5..].as_bytes()).ok()
        })
        .iter()
        .any(|signature| {
            let mut mac = Hmac::new(Sha1::new(), &secret.as_bytes());
            mac.input(&request.body());
            mac.result() == MacResult::new(&signature)
        })
}

#[cfg_attr(tarpaulin, skip)]
gateway!(|request, _| {
    let config = envy::from_env::<Config>()?;
    println!("{}", ::std::str::from_utf8(&request.body())?);
    if authenticated(&request, &config.github_webhook_secret) {
        if let Ok(Some(payload)) = request.payload::<github::Payload>() {
            println!("{:?}", payload);
        }
    } else {
        for metric in incr(
            "barbershop.fail",
            vec!["reason:invalid_authentication".into()],
        ) {
            println!("{}", metric);
        }
        eprintln!("recieved unauthenticated request");
    }

    Ok(lando::Response::new(()))
});

#[cfg(test)]
mod tests {
    use super::{authenticated, lando};

    #[test]
    fn missing_header_is_authenticated() {
        assert!(!authenticated(
            &lando::Request::new("{}".into()),
            &"secret".to_string()
        ))
    }
}
