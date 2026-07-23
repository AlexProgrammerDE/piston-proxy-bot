use piston_proxy_commands::ProxyProtocol;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProxyApiResponse {
    success: bool,
    http: Option<Vec<String>>,
    https: Option<Vec<String>>,
    socks4: Option<Vec<String>>,
    socks5: Option<Vec<String>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ProxyLists {
    http: Vec<String>,
    https: Vec<String>,
    socks4: Vec<String>,
    socks5: Vec<String>,
}

impl ProxyApiResponse {
    #[must_use]
    pub fn into_proxy_lists(self) -> Option<ProxyLists> {
        if !self.success {
            return None;
        }

        Some(ProxyLists {
            http: self.http?,
            https: self.https?,
            socks4: self.socks4?,
            socks5: self.socks5?,
        })
    }
}

impl ProxyLists {
    #[must_use]
    pub fn format(self, protocol: ProxyProtocol) -> String {
        match protocol {
            ProxyProtocol::Http => self.http.join("\n"),
            ProxyProtocol::Https => self.https.join("\n"),
            ProxyProtocol::Socks4 => self.socks4.join("\n"),
            ProxyProtocol::Socks5 => self.socks5.join("\n"),
            ProxyProtocol::All => [
                with_scheme("http", self.http),
                with_scheme("https", self.https),
                with_scheme("socks4", self.socks4),
                with_scheme("socks5", self.socks5),
            ]
            .join("\n"),
        }
    }
}

fn with_scheme(scheme: &str, proxies: Vec<String>) -> String {
    proxies
        .into_iter()
        .map(|proxy| format!("{scheme}://{proxy}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use piston_proxy_commands::ProxyProtocol;

    use super::{ProxyApiResponse, ProxyLists};

    fn proxy_lists() -> ProxyLists {
        ProxyLists {
            http: vec!["192.0.2.1:80".into(), "192.0.2.2:8080".into()],
            https: vec!["198.51.100.1:443".into()],
            socks4: vec!["203.0.113.1:1080".into()],
            socks5: vec!["203.0.113.2:1080".into()],
        }
    }

    #[test]
    fn formats_individual_and_combined_proxy_lists() {
        assert_eq!(
            proxy_lists().format(ProxyProtocol::Http),
            "192.0.2.1:80\n192.0.2.2:8080"
        );
        assert_eq!(
            proxy_lists().format(ProxyProtocol::All),
            concat!(
                "http://192.0.2.1:80\n",
                "http://192.0.2.2:8080\n",
                "https://198.51.100.1:443\n",
                "socks4://203.0.113.1:1080\n",
                "socks5://203.0.113.2:1080"
            )
        );
    }

    #[test]
    fn requires_a_successful_complete_api_response() {
        let failed: ProxyApiResponse = serde_json::from_value(serde_json::json!({
            "success": false
        }))
        .expect("the failure response should deserialize");
        assert_eq!(failed.into_proxy_lists(), None);

        let incomplete: ProxyApiResponse = serde_json::from_value(serde_json::json!({
            "success": true,
            "http": [],
            "https": [],
            "socks4": []
        }))
        .expect("the incomplete response should deserialize");
        assert_eq!(incomplete.into_proxy_lists(), None);
    }
}
