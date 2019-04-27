use reqwest::{Client, Error};
use serde_derive::Deserialize;

/// Pull request review payload
///
/// See Github's [docs](https://developer.github.com/v3/activity/events/types/#pullrequestevent)
/// for more information
#[derive(Deserialize, Debug)]
pub struct Payload {
    pub action: String,
    pub number: usize,
    pub pull_request: PullRequest,
}

impl Payload {
    /// Return `true` if pull request was closed,
    /// regardless of pull request merge status
    pub fn deletable(&self) -> bool {
        "closed" == self.action
    }

    /// Return full url of
    pub fn ref_url(&self) -> String {
        format!(
            "{repo_url}/git/refs/heads/{ref_name}",
            repo_url = self.pull_request.head.repo.url,
            ref_name = self.pull_request.head.branch
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct PullRequest {
    pub head: Ref,
    pub merged: bool,
}

#[derive(Deserialize, Debug)]
pub struct Ref {
    #[serde(rename = "ref")]
    pub branch: String,
    pub repo: Repository,
}

#[derive(Deserialize, Debug)]
pub struct Repository {
    /// api url for repo
    pub url: String,
    /// {owner}/{repo}
    pub full_name: String,
}

//// https://developer.github.com/v3/git/refs/#delete-a-reference
pub fn delete(token: &String, url: &String) -> Result<(), Error> {
    Client::new()
        .delete(url)
        .basic_auth("", Some(token.clone()))
        .send()
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::Payload;
    use lambda_http::{Request, RequestExt};

    const OPENED: &str = include_str!("../data/opened.json");
    const CLOSED: &str = include_str!("../data/closed.json");

    #[test]
    fn extracts_ref_url() {
        let mut req = Request::new(OPENED.into());
        req.headers_mut().insert(
            "Content-Type",
            "application/json".parse().expect("invalid header value"),
        );
        let payload = req
            .payload::<Payload>()
            .expect("unable to parse payload")
            .expect("expected some body");
        assert_eq!(
            payload.ref_url(),
            "https://api.github.com/repos/Codertocat/Hello-World/git/refs/heads/changes"
        );
    }

    #[test]
    fn closed_is_deletable() {
        let mut req = Request::new(CLOSED.into());
        req.headers_mut().insert(
            "Content-Type",
            "application/json".parse().expect("invalid header value"),
        );
        let payload = req
            .payload::<Payload>()
            .expect("unable to parse payload")
            .expect("expected some body");
        assert!(payload.deletable());
    }

    #[test]
    fn opened_is_not_deltable() {
        let mut req = Request::new(OPENED.into());
        req.headers_mut().insert(
            "Content-Type",
            "application/json".parse().expect("invalid header value"),
        );
        let payload = req
            .payload::<Payload>()
            .expect("unable to parse payload")
            .expect("expected some body");
        assert!(!payload.deletable());
    }
}
