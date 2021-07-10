use reqwest::{Client, Proxy};
use std::env::args;
use std::sync::Arc;
use std::time::Duration;

const PROXIES_URL: &str =
    "https://raw.githubusercontent.com/clarketm/proxy-list/master/proxy-list-raw.txt";

#[tokio::main]
async fn main() {
    let target_url: Arc<String> =
        Arc::from(args().nth(1).unwrap_or("http://example.com".to_string()));

    let proxies_raw = reqwest::get(PROXIES_URL)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let proxies = proxies_raw
        .trim()
        .split('\n')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let mut futures = Vec::with_capacity(proxies.len());
    for proxy in proxies.into_iter() {
        let target_url = Arc::clone(&target_url);
        futures.push(tokio::spawn(async move {
            let req_proxy = Proxy::http(format!("http://{}", proxy)).unwrap();
            let client = Client::builder()
                .proxy(req_proxy)
                .timeout(Duration::from_secs(30))
                .connect_timeout(Duration::from_secs(30))
                .build()
                .unwrap();

            if let Ok(resp) = client.get(target_url.as_str()).send().await {
                if resp.status().is_success() {
                    return Some(proxy);
                }
            };

            None
        }))
    }

    let mut good_proxies = Vec::new();
    for future in futures {
        if let Some(proxy) = future.await.unwrap() {
            good_proxies.push(proxy);
        }
    }

    println!("{}", good_proxies.join("\n"))
}
