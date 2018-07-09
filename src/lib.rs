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

// Third party
use crypto::hmac::Hmac;
use crypto::mac::{Mac, MacResult};
use crypto::sha1::Sha1;
use hex::FromHex;
use lando::{RequestExt, Response};

mod github;
mod metric;

#[derive(Deserialize)]
struct Config {
    github_token: String,
    github_webhook_secret: String,
}

fn header<'a>(request: &'a lando::Request, name: &str) -> Option<&'a str> {
    request
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
}

/// Return true if webhook was authenticated, false otherwise
fn authenticated(request: &lando::Request, secret: &String) -> bool {
    header(request, "X-Hub-Signature")
        .and_then(|value| {
            // strip off `sha1=` and get hex bytes
            Vec::from_hex(value[5..].as_bytes()).ok()
        })
        .iter()
        .any(|signature| {
            let mut mac = Hmac::new(Sha1::new(), &secret.as_bytes());
            mac.input(&request.body());
            mac.result() == MacResult::new(&signature)
        })
}

fn incr_trim(repo: &String, branch: &String) -> Option<String> {
    metric::incr(
        "barbershop.trim",
        vec![format!("repo:{}", repo), format!("branch:{}", branch)],
    )
}

fn incr_auth_fail() -> Option<String> {
    metric::incr(
        "barbershop.fail",
        vec!["reason:invalid_authentication".into()],
    )
}

fn incr_trim_fail(reason: &String, repo: &String, branch: &String) -> Option<String> {
    metric::incr(
        "barbershop.fail",
        vec![
            format!("repo:{}", repo),
            format!("branch:{}", branch),
            format!("reason:{}", reason),
        ],
    )
}

#[cfg_attr(tarpaulin, skip)]
gateway!(|request, _| {
    let config = envy::from_env::<Config>()?;
    if !authenticated(&request, &config.github_webhook_secret) {
        for metric in incr_auth_fail() {
            println!("{}", metric);
        }
        return Ok(Response::builder().status(401).body(())?);
    }
    match request.payload::<github::Payload>() {
        Ok(Some(ref payload)) if payload.deletable() => {
            if let Err(e) = github::delete(&config.github_token.clone(), &payload.ref_url()) {
                for metric in incr_trim_fail(
                    &e.to_string(),
                    &payload.pull_request.head.repo.full_name,
                    &payload.pull_request.head.branch,
                ) {
                    println!("{}", metric);
                }
                return Ok(Response::builder().status(400).body(())?);
            }

            for metric in incr_trim(
                &payload.pull_request.head.repo.full_name,
                &payload.pull_request.head.branch,
            ) {
                println!("{}", metric);
            }
        }
        _ => (),
    }

    Ok(Response::new(()))
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
