use anyhow::Result;
use url::Url;

pub fn parse_without_user_pass(url: &str) -> Result<String> {
    let parsed = Url::parse(&url)?;
    let scheme = parsed.scheme();
    let path = parsed.path();
    let port = parsed.port();
    let host = parsed.host();

    let host = match host {
        None => "[NONE!]".to_string(),
        Some(host) => host.to_string(),
    };

    let port = match port {
        None => "5432".to_string(),
        Some(port) => port.to_string(),
    };

    Ok(format!("{scheme}://@{host}:{port}{path}"))
}

#[test]
fn test_parse_without_user_pass() {
    assert_eq!(
        parse_without_user_pass("postgres://username:password@host_name:9999/database_name")
            .unwrap(),
        "postgres://@host_name:9999/database_name",
    );
}
