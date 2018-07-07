// Third party
use reqwest::Client;

#[derive(Deserialize, Debug)]
pub struct Payload {
  pub action: String,
  pub number: usize,
  pub pull_request: PullRequest,
}

#[derive(Deserialize, Debug)]
pub struct PullRequest {
  pub url: String,
  pub html_url: String,
  pub title: String,
  pub state: String,
  pub body: Option<String>,
  pub head: Ref,
}

#[derive(Deserialize, Debug)]
pub struct Ref {
  #[serde(rename = "ref")]
  pub branch: String,
}

//// https://developer.github.com/v3/git/refs/#delete-a-reference
pub fn delete(token: &String, url: &String) -> Option<()> {
  Client::new()
    .expect("failed to create client")
    .delete(url)
    .expect("failed to parse url")
    .basic_auth("", Some(token.clone()))
    .send()
    .map(|_| ())
    .ok()
}
