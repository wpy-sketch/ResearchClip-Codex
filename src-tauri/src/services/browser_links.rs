pub fn public_url(value: &str) -> Result<reqwest::Url, String> {
    if value.len() > 4096 || value.chars().any(char::is_control) {
        return Err(String::from("网页地址过长或包含控制字符。"));
    }
    let u = reqwest::Url::parse(value)
        .map_err(|_| String::from("请输入完整的HTTP或HTTPS网页地址。"))?;
    let host = u
        .host_str()
        .unwrap_or_default()
        .trim_end_matches('.')
        .to_ascii_lowercase();
    let ip = host
        .trim_matches(['[', ']'])
        .parse::<std::net::IpAddr>()
        .ok();
    let local = match ip {
        Some(std::net::IpAddr::V4(a)) => {
            a.is_private()
                || a.is_loopback()
                || a.is_link_local()
                || a.is_unspecified()
                || a.is_broadcast()
                || a.is_multicast()
        }
        Some(std::net::IpAddr::V6(_)) => true,
        None => false,
    };
    if !matches!(u.scheme(), "https" | "http")
        || !u.username().is_empty()
        || u.password().is_some()
        || host.is_empty()
        || !host.contains('.')
        || host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || local
    {
        return Err(String::from(
            "仅允许公开HTTP(S)网页；本地文件、应用协议、内网及带凭据地址不可在此打开。",
        ));
    }
    Ok(u)
}
