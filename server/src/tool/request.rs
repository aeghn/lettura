use crate::model::ProxyConfig;

pub fn create_client(local_proxy: Option<&ProxyConfig>) -> reqwest::Client {
    let client_builder = reqwest::Client::builder();

    if let Some(proxy) = local_proxy {
        let mut scheme = String::from("http://");

        scheme.push_str(&proxy.ip.to_string());
        scheme.push_str(":");
        scheme.push_str(&proxy.port.to_string());

        return client_builder
            .proxy(reqwest::Proxy::all(scheme).unwrap())
            .user_agent("curl")
            .build()
            .unwrap();
    }

    client_builder.build().unwrap()
}
