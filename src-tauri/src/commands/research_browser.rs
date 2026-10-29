//! Unprivileged native child view. Remote pages never receive an application capability.
use super::{CommandError, library::check};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalSize, Rect, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
    webview::{NewWindowResponse, PageLoadEvent, PermissionResponse},
};
const LABEL: &str = "research-browser";
const HOME: &str = "https://www.google.com/";
#[cfg(target_os = "windows")]
fn embedded_builder<'a>(
    builder: WebviewWindowBuilder<'a, tauri::Wry, AppHandle>,
    parent: &WebviewWindow,
) -> Result<WebviewWindowBuilder<'a, tauri::Wry, AppHandle>, CommandError> {
    Ok(builder.parent_raw(parent.hwnd().map_err(|_| fail("无法取得主窗口。"))?))
}
#[cfg(not(target_os = "windows"))]
fn embedded_builder<'a>(
    _builder: WebviewWindowBuilder<'a, tauri::Wry, AppHandle>,
    _parent: &WebviewWindow,
) -> Result<WebviewWindowBuilder<'a, tauri::Wry, AppHandle>, CommandError> {
    Err(fail("应用内网页区域当前仅支持Windows；请使用外部浏览器。"))
}
#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    url: String,
    title: String,
    loading: bool,
    notice: String,
    visible: bool,
}
#[derive(Default)]
pub struct BrowserHost {
    operation: Mutex<()>,
    snapshot: Mutex<Snapshot>,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "camelCase", deny_unknown_fields)]
pub enum BrowserRequest {
    Show { bounds: Bounds },
    Hide {},
    Navigate { url: String },
    Back {},
    Forward {},
    Reload {},
    Stop {},
    Zoom { factor: f64 },
    Status {},
    Close {},
}
fn fail(message: impl Into<String>) -> CommandError {
    CommandError {
        code: "RESEARCH_BROWSER_ERROR",
        message: message.into(),
    }
}
fn web_url(value: &str) -> Result<reqwest::Url, CommandError> {
    crate::services::browser_links::public_url(value).map_err(fail)
}

fn bounds(b: Bounds, size: PhysicalSize<u32>) -> Result<Rect, CommandError> {
    if [b.x, b.y, b.width, b.height].iter().any(|n| !n.is_finite())
        || b.x < 0.
        || b.y < 0.
        || b.width < 32.
        || b.height < 32.
        || b.x + b.width > f64::from(size.width) + 2.
        || b.y + b.height > f64::from(size.height) + 2.
    {
        return Err(fail("网页区域尚不可见或超出主窗口，请重新调整布局。"));
    }
    Ok(Rect {
        position: PhysicalPosition::new(b.x.round() as i32, b.y.round() as i32).into(),
        size: PhysicalSize::new(b.width.round() as u32, b.height.round() as u32).into(),
    })
}
fn notice(app: &AppHandle, message: &str) {
    if let Ok(mut s) = app.state::<Arc<BrowserHost>>().snapshot.lock() {
        s.notice = message.into();
    }
}
fn execute(
    window: WebviewWindow,
    app: AppHandle,
    request: BrowserRequest,
) -> Result<Snapshot, CommandError> {
    let host = app.state::<Arc<BrowserHost>>();
    let _serial = host
        .operation
        .lock()
        .map_err(|_| fail("网页服务锁不可用。"))?;
    if let BrowserRequest::Show { bounds: b } = request {
        let r = bounds(
            b,
            window
                .inner_size()
                .map_err(|_| fail("无法读取窗口尺寸。"))?,
        )?;
        let child = if let Some(v) = app.get_webview_window(LABEL) {
            v
        } else {
            let navigation = app.clone();
            let popup = app.clone();
            let builder =
                WebviewWindowBuilder::new(&app, LABEL, WebviewUrl::External(web_url(HOME)?))
                    .visible(false)
                    .decorations(false)
                    .shadow(false)
                    .skip_taskbar(true)
                    .focused(false)
                    .incognito(true)
                    .devtools(false)
                    .on_navigation(move |url| {
                        let allowed = web_url(url.as_str()).is_ok();
                        if !allowed {
                            notice(&navigation, "已阻止本地文件、应用协议或内网跳转。");
                        }
                        allowed
                    })
                    .on_new_window(move |url, _| {
                        if web_url(url.as_str()).is_ok() {
                            let a = popup.clone();
                            tauri::async_runtime::spawn_blocking(move || {
                                if let Some(v) = a.get_webview_window(LABEL)
                                    && v.navigate(url).is_err()
                                {
                                    notice(&a, "新链接未能打开，请在地址栏重试。");
                                }
                            });
                        } else {
                            notice(&popup, "已阻止不支持的新窗口地址。");
                        }
                        NewWindowResponse::Deny
                    })
                    .on_download(|v, _| {
                        notice(
                            v.app_handle(),
                            "应用内浏览器不自动下载文件，请使用外部Chrome下载。",
                        );
                        false
                    })
                    .on_permission_request(|v, _| {
                        notice(
                            v.app_handle(),
                            "网页设备与通知权限已拒绝；需要这些能力时请使用外部Chrome。",
                        );
                        PermissionResponse::Deny
                    })
                    .on_document_title_changed(|v, title| {
                        if let Ok(mut s) = v.state::<Arc<BrowserHost>>().snapshot.lock() {
                            s.title = title.chars().take(200).collect();
                        }
                    })
                    .on_page_load(|v, payload| {
                        if let Ok(mut s) = v.state::<Arc<BrowserHost>>().snapshot.lock() {
                            s.url = payload.url().to_string();
                            s.loading = matches!(payload.event(), PageLoadEvent::Started);
                            if s.loading {
                                s.notice.clear();
                            }
                        }
                    });
            embedded_builder(builder, &window)?
                .build()
                .map_err(|_| fail("无法创建应用内浏览器，请检查WebView2运行时。"))?
        };
        child
            .set_position(r.position)
            .and_then(|_| child.set_size(r.size))
            .and_then(|_| child.show())
            .map_err(|_| fail("网页区域布局更新失败。"))?;
        host.snapshot
            .lock()
            .map_err(|_| fail("网页状态不可用。"))?
            .visible = true;
    } else if matches!(request, BrowserRequest::Hide {} | BrowserRequest::Close {}) {
        if let Some(v) = app.get_webview_window(LABEL) {
            if matches!(request, BrowserRequest::Close {}) {
                v.close()
            } else {
                v.hide()
            }
            .map_err(|_| fail("无法隐藏或关闭网页。"))?;
        }
        let mut s = host.snapshot.lock().map_err(|_| fail("网页状态不可用。"))?;
        s.visible = false;
        if matches!(request, BrowserRequest::Close {}) {
            *s = Snapshot::default();
        }
    } else if !matches!(request, BrowserRequest::Status {}) {
        let v = app
            .get_webview_window(LABEL)
            .ok_or_else(|| fail("请先打开应用内浏览器。"))?;
        match request {
            BrowserRequest::Navigate { url } => v.navigate(web_url(url.trim())?),
            BrowserRequest::Back {} => v.eval("history.back()"),
            BrowserRequest::Forward {} => v.eval("history.forward()"),
            BrowserRequest::Reload {} => v.reload(),
            BrowserRequest::Stop {} => v.eval("window.stop()"),
            BrowserRequest::Zoom { factor } => {
                if !factor.is_finite() || !(0.5..=2.).contains(&factor) {
                    return Err(fail("网页缩放范围为50%–200%。"));
                }
                v.set_zoom(factor)
            }
            _ => unreachable!(),
        }
        .map_err(|_| fail("网页操作失败，请重试或使用外部浏览器。"))?;
    }
    if let Some(view) = app.get_webview_window(LABEL)
        && let Ok(url) = view.url()
        && web_url(url.as_str()).is_ok()
    {
        host.snapshot
            .lock()
            .map_err(|_| fail("网页状态不可用。"))?
            .url = url.to_string();
    }
    host.snapshot
        .lock()
        .map(|s| s.clone())
        .map_err(|_| fail("网页状态不可用。"))
}
#[tauri::command]
pub async fn research_browser(
    window: WebviewWindow,
    app: AppHandle,
    request: BrowserRequest,
) -> Result<Snapshot, CommandError> {
    check(&window)?;
    tauri::async_runtime::spawn_blocking(move || execute(window, app, request))
        .await
        .map_err(|_| fail("网页后台任务失败。"))?
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn web_addresses_are_bounded_and_unprivileged() {
        for s in [
            HOME,
            "https://example.org/paper?q=研究#p2",
            "http://example.com/",
        ] {
            assert!(web_url(s).is_ok(), "{s}");
        }
        for s in [
            "file:///C:/x",
            "javascript:alert(1)",
            "data:text/html,x",
            "tauri://localhost/",
            "https://u:p@example.com",
            "http://127.0.0.1:32132/api/",
            "https://localhost./",
            "https://a.local/",
            "http://192.168.1.1/",
            "http://[::1]/",
            "http://2130706433/",
            "https://example.com/\n",
        ] {
            assert!(web_url(s).is_err(), "{s}");
        }
    }
    #[test]
    fn bounds_reject_hidden_nonfinite_and_outside_window() {
        let size = PhysicalSize::new(1200, 800);
        assert!(
            bounds(
                Bounds {
                    x: 100.,
                    y: 200.,
                    width: 900.,
                    height: 500.
                },
                size
            )
            .is_ok()
        );
        for b in [
            Bounds {
                x: -1.,
                y: 0.,
                width: 100.,
                height: 100.,
            },
            Bounds {
                x: 0.,
                y: 0.,
                width: f64::NAN,
                height: 100.,
            },
            Bounds {
                x: 0.,
                y: 0.,
                width: 1300.,
                height: 100.,
            },
            Bounds {
                x: 0.,
                y: 0.,
                width: 0.,
                height: 100.,
            },
        ] {
            assert!(bounds(b, size).is_err());
        }
    }
    #[test]
    fn requests_reject_scripts_and_unknown_fields() {
        for s in [
            r#"{"operation":"eval","script":"x"}"#,
            r#"{"operation":"back","url":"https://example.com"}"#,
            r#"{"operation":"show","bounds":{"x":0,"y":0,"width":100,"height":100,"script":"x"}}"#,
        ] {
            assert!(serde_json::from_str::<BrowserRequest>(s).is_err());
        }
    }
}
