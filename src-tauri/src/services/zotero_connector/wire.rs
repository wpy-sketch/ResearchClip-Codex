//! A deliberately small HTTP surface using the existing pinned httparse parser.
use super::model::MAX_BODY;
use std::io::{Read, Write};
pub struct Request {
    pub path: String,
    pub authorization: String,
    pub body: Vec<u8>,
}
pub fn read(stream: &mut impl Read, port: u16) -> Result<Request, &'static str> {
    let mut bytes = Vec::new();
    let (head_len, path, authorization, len) = loop {
        if bytes.len() > 8192 + MAX_BODY {
            return Err("请求超过预算。");
        }
        let mut headers = [httparse::EMPTY_HEADER; 32];
        let mut req = httparse::Request::new(&mut headers);
        match req.parse(&bytes).map_err(|_| "HTTP头无效。")? {
            httparse::Status::Partial => {
                if bytes.len() > 8192 {
                    return Err("HTTP头超限。");
                }
            }
            httparse::Status::Complete(n) => {
                if n > 8192
                    || req.method != Some("POST")
                    || req.version != Some(1)
                    || !matches!(req.path, Some("/pair" | "/message"))
                {
                    return Err("HTTP方法、路径或版本未授权。");
                }
                let mut names = std::collections::HashSet::new();
                let mut authorization = None;
                let mut length = None;
                let mut host = false;
                let mut content = false;
                for h in req.headers.iter() {
                    let name = h.name.to_ascii_lowercase();
                    if !names.insert(name.clone())
                        || matches!(
                            name.as_str(),
                            "origin" | "referer" | "transfer-encoding" | "upgrade"
                        )
                    {
                        return Err("网页来源、代理或重复头未授权。");
                    }
                    let value = std::str::from_utf8(h.value).map_err(|_| "HTTP值不是UTF-8。")?;
                    match name.as_str() {
                        "host" => host = value == format!("127.0.0.1:{port}"),
                        "content-type" => {
                            content = value
                                .split(';')
                                .next()
                                .is_some_and(|s| s.trim().eq_ignore_ascii_case("application/json"))
                        }
                        "content-length" => {
                            if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
                                return Err("长度格式无效。");
                            }
                            length = Some(value.parse::<usize>().map_err(|_| "长度溢出。")?);
                        }
                        "authorization" => {
                            authorization = value.strip_prefix("Bearer ").map(str::to_owned)
                        }
                        _ => {}
                    }
                }
                let len = length.ok_or("缺少长度。");
                let auth = authorization.ok_or("缺少能力授权。");
                let len = len?;
                if !host || !content || !(1..=MAX_BODY).contains(&len) {
                    return Err("主机、JSON类型或大小未授权。");
                }
                let auth = auth?;
                if auth.len() != 64
                    || !auth
                        .bytes()
                        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
                {
                    return Err("授权格式无效。");
                }
                break (n, req.path.unwrap().to_owned(), auth, len);
            }
        }
        let mut chunk = [0; 2048];
        let n = stream.read(&mut chunk).map_err(|_| "读取失败或超时。");
        let n = n?;
        if n == 0 {
            return Err("请求提前结束。");
        }
        bytes.extend_from_slice(&chunk[..n]);
    };
    if bytes.len() > head_len + len {
        return Err("不允许管线或附加消息。");
    }
    while bytes.len() < head_len + len {
        let remain = head_len + len - bytes.len();
        let mut chunk = [0; 2048];
        let n = stream
            .read(&mut chunk[..remain.min(2048)])
            .map_err(|_| "正文读取失败或超时。");
        let n = n?;
        if n == 0 {
            return Err("正文提前结束。");
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
    Ok(Request {
        path,
        authorization,
        body: bytes[head_len..].to_vec(),
    })
}
pub fn respond(
    stream: &mut impl Write,
    status: u16,
    value: &serde_json::Value,
) -> std::io::Result<()> {
    let body = serde_json::to_vec(value)?;
    write!(
        stream,
        "HTTP/1.1 {status} Response\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\n\r\n",
        body.len()
    )?;
    stream.write_all(&body)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn packet(extra: &str, body: &str) -> Vec<u8> {
        format!("POST /message HTTP/1.1\r\nHost: 127.0.0.1:32128\r\nContent-Type: application/json\r\nAuthorization: Bearer {}\r\nContent-Length: {}\r\n{extra}\r\n{body}","a".repeat(64),body.len()).into_bytes()
    }
    #[test]
    fn parses_only_bounded_local_json_and_never_cors() {
        let r = read(&mut packet("", "{}").as_slice(), 32128).unwrap();
        assert_eq!(r.body, b"{}");
        for extra in [
            "Origin: https://evil.invalid\r\n",
            "Content-Length: 2\r\n",
            "Transfer-Encoding: chunked\r\n",
            "Upgrade: websocket\r\n",
        ] {
            assert!(read(&mut packet(extra, "{}").as_slice(), 32128).is_err());
        }
        assert!(read(&mut packet("", "{}").as_slice(), 1).is_err());
    }
    #[test]
    fn truncation_pipeline_wrong_method_and_unbounded_headers_fail() {
        let mut b = packet("", "{}");
        b.pop();
        assert!(read(&mut b.as_slice(), 32128).is_err());
        let mut b = packet("", "{}");
        b.extend_from_slice(b"GET / HTTP/1.1");
        assert!(read(&mut b.as_slice(), 32128).is_err());
        let b = packet("", "{}");
        let b = String::from_utf8(b).unwrap().replacen("POST", "GET", 1);
        assert!(read(&mut b.as_bytes(), 32128).is_err());
        assert!(read(&mut vec![b'x'; 9000].as_slice(), 32128).is_err());
    }
}
