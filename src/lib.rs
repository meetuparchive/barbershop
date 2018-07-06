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
use crypto::mac::Mac;
use crypto::mac::MacResult;
use crypto::sha1::Sha1;
use hex::FromHex;
use lando::RequestExt;

mod github;

#[derive(Deserialize)]
struct Config {
    github_token: String,
    github_webhook_secret: String,
}

fn authenticated(request: &lando::Request, secret: &String) -> bool {
    request
        .headers()
        .get("X-Hub-Signature")
        .iter()
        .filter_map(|value| {
            // strip off `sha1=` and get hex bytes
            Vec::from_hex(value.to_str().expect("invalid header")[5..].as_bytes()).ok()
        })
        .any(|signature| {
            let mut mac = Hmac::new(Sha1::new(), &secret.as_bytes());
            mac.input(&request.body());
            mac.result() == MacResult::new(&signature)
        })
}

gateway!(|request, _| {
    let config = envy::from_env::<Config>()?;
    if authenticated(&request, &config.github_webhook_secret) {
        if let Ok(Some(payload)) = request.payload::<github::Payload>() {
            println!("{:?}", payload);
        }
    } else {
        eprintln!("recieved unauthenticated request");
    }

    Ok(lando::Response::new(()))
});
