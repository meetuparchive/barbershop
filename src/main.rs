use crypto::{
    hmac::Hmac,
    mac::{Mac, MacResult},
    sha1::Sha1,
};
use hex::FromHex;
use lambda_http::{lambda, IntoResponse, Request, RequestExt, Response};
use lambda_runtime::{error::HandlerError, Context};
use serde_derive::Deserialize;

mod github;
mod metric;

#[derive(Deserialize)]
struct Config {
    github_token: String,
    github_webhook_secret: String,
}

fn header<'a>(request: &'a Request, name: &str) -> Option<&'a str> {
    request.headers().get(name).and_then(|value| value.to_str().ok())
}

/// Return true if webhook was authenticated, false otherwise
fn authenticated(request: &Request, secret: &str) -> bool {
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

fn incr_trim(repo: &str, branch: &str) -> Option<String> {
    metric::incr(
        "barbershop.trim",
        vec![format!("repo:{}", repo), format!("branch:{}", branch)],
    )
}

fn incr_auth_fail() -> Option<String> {
    metric::incr("barbershop.fail", vec!["reason:invalid_authentication".into()])
}

fn incr_trim_fail(reason: &str, repo: &str, branch: &str) -> Option<String> {
    metric::incr(
        "barbershop.fail",
        vec![
            format!("repo:{}", repo),
            format!("branch:{}", branch),
            format!("reason:{}", reason),
        ],
    )
}

fn main() {
    lambda!(trim)
}

fn trim(request: Request, _ctx: Context) -> Result<impl IntoResponse, HandlerError> {
    let config = envy::from_env::<Config>().map_err(|e| HandlerError::from(e.to_string().as_str()))?;

    if !authenticated(&request, &config.github_webhook_secret) {
        if let Some(metric) = incr_auth_fail() {
            println!("{}", metric);
        }
        return Ok(Response::builder()
            .status(401)
            .body(())
            .map_err(|e| HandlerError::from(e.to_string().as_str()))?);
    }

    if let Ok(Some(ref payload)) = request.payload::<github::Payload>() {
        if payload.deletable() {
            if let Err(e) = github::delete(&config.github_token.clone(), &payload.ref_url()) {
                if let Some(metric) = incr_trim_fail(
                    &e.to_string(),
                    &payload.pull_request.head.repo.full_name,
                    &payload.pull_request.head.branch,
                ) {
                    println!("{}", metric);
                }
                return Ok(Response::builder()
                    .status(400)
                    .body(())
                    .map_err(|e| HandlerError::from(e.to_string().as_str()))?);
            }

            if let Some(metric) = incr_trim(
                &payload.pull_request.head.repo.full_name,
                &payload.pull_request.head.branch,
            ) {
                println!("{}", metric);
            }
        }
    }

    Ok(Response::new(()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_header_is_authenticated() {
        assert!(!authenticated(&Request::new("{}".into()), &"secret".to_string()))
    }
}
