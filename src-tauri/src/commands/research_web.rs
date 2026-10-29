use super::{CommandError, library::check};
use tauri::{AppHandle, WebviewWindow};
use tauri_plugin_opener::OpenerExt;
#[derive(serde::Deserialize)]
#[serde(tag = "operation", rename_all = "camelCase", deny_unknown_fields)]
pub enum WebRequest {
    Translate { text: String },
    GoogleChrome {},
}
fn translation_url(text: &str) -> Result<String, String> {
    if text.trim().is_empty() || text.len() > 600 || text.chars().any(char::is_control) {
        return Err("划词翻译限600字节以内的非空文字；不会发送整个PDF。".into());
    }
    let mut url =
        reqwest::Url::parse("https://translate.google.com/").map_err(|_| "翻译入口不可用。")?;
    url.query_pairs_mut().extend_pairs([
        ("sl", "auto"),
        ("tl", "zh-CN"),
        ("text", text),
        ("op", "translate"),
    ]);
    Ok(url.into())
}
#[tauri::command]
pub fn open_research_web(
    window: WebviewWindow,
    app: AppHandle,
    request: WebRequest,
) -> Result<(), CommandError> {
    check(&window)?;
    let fail = |message| CommandError {
        code: "RESEARCH_WEB_ERROR",
        message,
    };
    let (url, browser) = match request {
        WebRequest::Translate { text } => (translation_url(&text).map_err(fail)?, None),
        WebRequest::GoogleChrome {} => ("https://www.google.com/".to_string(), Some("chrome")),
    };
    app.opener()
        .open_url(url, browser)
        .map_err(|_| fail("无法打开浏览器。Google入口需要已安装Chrome；请检查安装及系统关联，原编辑会话保持不变。".into()))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chrome_request_has_no_url_program_or_profile_override() {
        assert!(serde_json::from_str::<WebRequest>(r#"{"operation":"googleChrome"}"#).is_ok());
        for field in ["url", "program", "profile"] {
            let value = serde_json::json!({"operation":"googleChrome",field:"file:///untrusted"});
            assert!(serde_json::from_value::<WebRequest>(value).is_err());
        }
    }
    #[test]
    fn translation_is_explicit_bounded_encoded_and_fixed_origin() {
        let s = "公式 a+b & x=1 中文";
        let url = reqwest::Url::parse(&translation_url(s).unwrap()).unwrap();
        assert_eq!(url.host_str(), Some("translate.google.com"));
        assert_eq!(url.query_pairs().find(|(k, _)| k == "text").unwrap().1, s);
        for bad in ["".to_string(), "a".repeat(601), "a\0b".into()] {
            assert!(translation_url(&bad).is_err());
        }
        assert!(
            serde_json::from_str::<WebRequest>(
                r#"{"operation":"translate","text":"a","url":"file:///x"}"#
            )
            .is_err()
        );
    }
}
