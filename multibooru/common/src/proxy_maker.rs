/// Wrap the given [`reqwest::ClientBuilder`] with a proxy.
pub fn with_proxy(client: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
    let proxy = std::env::var("PROXY_URL")
        .ok()
        .and_then(|proxy| reqwest::Proxy::all(&proxy).ok());
    if let Some(proxy) = proxy {
        client.proxy(proxy)
    } else {
        client
    }
}

/// Wrap the given [`reqwest::ClientBuilder`] with a proxy suitable for downloading media assets.
/// 
/// Some proxies are not suitable for downloading media assets, and may incur significant costs if used as such.
/// These proxies should only be used for downloading the JSON and HTML files that contain links to media assets.
pub fn with_media_proxy(client: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
    // TODO: currently this is just a copy of with_proxy, but it should be changed to use a different environment variable.
    with_proxy(client)
}