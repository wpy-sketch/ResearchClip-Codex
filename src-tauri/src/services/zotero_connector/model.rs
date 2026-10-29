//! Session capabilities and scoped Zotero identities, never global resource IDs.
use super::super::zotero_local::{instance_identity, valid_item_key};
use serde::{Deserialize, Serialize};
pub const PLUGIN_ID: &str = "researchclip-connector@wpy-sketch.github.io";
pub const PLUGIN_VERSION: &str = "1.0.1";
pub const PROTOCOL: u32 = 1;
pub const MAX_BODY: usize = 64 * 1024;
pub fn id(value: &str) -> bool {
    crate::storage::files::id_valid(value)
}
pub fn text(value: &str, max: usize) -> bool {
    value.len() <= max && !value.chars().any(char::is_control)
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Identity {
    pub library_type: String,
    pub library_id: u64,
    pub local_library_id: Option<u64>,
    pub item_key: String,
    pub attachment_key: String,
}
impl Identity {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !matches!(self.library_type.as_str(), "user" | "group")
            || self.library_id > 9_007_199_254_740_991
            || self.local_library_id == Some(0)
            || self
                .local_library_id
                .is_some_and(|v| v > 9_007_199_254_740_991)
            || self.library_type == "user" && self.library_id != 0
            || self.library_type == "group" && self.library_id == 0
            || !valid_item_key(&self.item_key)
            || !valid_item_key(&self.attachment_key)
            || self.item_key == self.attachment_key
        {
            return Err("文献、附件或资料库身份无效。");
        }
        Ok(())
    }
    pub fn same_api_scope(&self, s: &super::super::zotero_annotations::AnnotationSource) -> bool {
        self.library_type == s.library_type
            && (self.library_type == "user" || self.library_id == s.library_id)
            && self.item_key == s.item_key
            && self.attachment_key == s.attachment_key
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Metadata {
    pub title: String,
    pub authors: Vec<String>,
    pub year: Option<i32>,
    pub doi: String,
}
impl Metadata {
    fn validate(&self) -> bool {
        !self.title.trim().is_empty()
            && text(&self.title, 8000)
            && self.authors.len() <= 200
            && self
                .authors
                .iter()
                .all(|a| !a.trim().is_empty() && text(a, 2000))
            && self.year.is_none_or(|y| (1..=9999).contains(&y))
            && text(&self.doi, 512)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Target {
    pub page_index: Option<u32>,
    pub annotation_key: Option<String>,
}
impl Target {
    pub fn validate(&self) -> bool {
        self.page_index.is_none_or(|p| p < 2000)
            && self.annotation_key.as_deref().is_none_or(valid_item_key)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Hello {
    pub version: u32,
    pub plugin_id: String,
    pub plugin_version: String,
    pub zotero_version: String,
    pub server_id: Option<String>,
}
impl Hello {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.version != PROTOCOL
            || self.plugin_id != PLUGIN_ID
            || self.plugin_version != PLUGIN_VERSION
            || self.zotero_version != "10.0.6"
            || self
                .server_id
                .as_deref()
                .is_some_and(|s| instance_identity(s).is_err())
        {
            return Err("插件、协议或Zotero版本尚未核实；未建立连接。");
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Envelope {
    pub version: u32,
    pub session_id: String,
    pub request_id: String,
    pub sequence: u64,
    pub message: Message,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Message {
    Open {
        identity: Box<Identity>,
        path: String,
        metadata: Box<Metadata>,
        target: Target,
    },
    Poll {
        acknowledgements: Vec<Acknowledgement>,
    },
    Changed {
        changes: Vec<Change>,
    },
    Disconnect {},
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Acknowledgement {
    pub id: String,
    pub success: bool,
    pub code: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Change {
    pub library_type: String,
    pub library_id: u64,
    pub item_key: String,
    pub attachment_key: Option<String>,
    pub action: String,
}
impl Envelope {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.version != PROTOCOL
            || self.sequence == 0
            || self.sequence > 9_007_199_254_740_991
            || !id(&self.session_id)
            || !id(&self.request_id)
        {
            return Err("协议或消息身份无效。");
        }
        match &self.message {
            Message::Open {
                identity,
                path,
                metadata,
                target,
            } => {
                identity.validate()?;
                if path.is_empty()
                    || !text(path, 32760)
                    || !metadata.validate()
                    || !target.validate()
                {
                    return Err("PDF打开参数、元数据或位置无效。");
                }
            }
            Message::Poll { acknowledgements } => {
                if acknowledgements.len() > 32
                    || acknowledgements
                        .iter()
                        .any(|a| !id(&a.id) || !text(&a.code, 120))
                {
                    return Err("反向定位回执无效。");
                }
            }
            Message::Changed { changes } => {
                if changes.is_empty()
                    || changes.len() > 100
                    || changes.iter().any(|c| {
                        !matches!(c.library_type.as_str(), "user" | "group")
                            || c.library_id > 9_007_199_254_740_991
                            || c.library_type == "user" && c.library_id != 0
                            || c.library_type == "group" && c.library_id == 0
                            || !valid_item_key(&c.item_key)
                            || c.attachment_key
                                .as_deref()
                                .is_some_and(|k| !valid_item_key(k))
                            || !matches!(
                                c.action.as_str(),
                                "add" | "modify" | "delete" | "trash" | "refresh"
                            )
                    })
                {
                    return Err("变化通知范围无效。");
                }
            }
            Message::Disconnect {} => {}
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OpenContext {
    pub connection_id: String,
    pub request_id: String,
    pub identity: Identity,
    pub metadata: Metadata,
    pub target: Target,
    pub server_id: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReverseCommand {
    pub id: String,
    pub identity: Identity,
    pub target: Target,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedNotice {
    pub connection_id: String,
    pub instance_id: Option<String>,
    pub changes: Vec<Change>,
}
pub type OpenGrant = (OpenContext, std::sync::Arc<std::sync::atomic::AtomicBool>);
#[cfg(test)]
mod tests {
    use super::*;
    fn identity() -> Identity {
        Identity {
            library_type: "user".into(),
            library_id: 0,
            local_library_id: Some(1),
            item_key: "PARENT01".into(),
            attachment_key: "ATTACH01".into(),
        }
    }
    #[test]
    fn identity_separates_api_alias_group_and_local_ids() {
        let a = identity();
        a.validate().unwrap();
        let mut b = a.clone();
        b.library_type = "group".into();
        assert!(b.validate().is_err());
        b.library_id = 7;
        b.validate().unwrap();
        assert_ne!(a, b);
        b.attachment_key = b.item_key.clone();
        assert!(b.validate().is_err());
    }
    #[test]
    fn unknown_commands_and_injected_fields_are_not_operations() {
        for message in [
            r#"{"version":1,"sequence":1,"sessionId":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","requestId":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","message":{"type":"execute","command":"powershell"}}"#,
            r#"{"version":1,"sequence":1,"sessionId":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","requestId":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","message":{"type":"disconnect","sql":"DELETE"}}"#,
        ] {
            assert!(serde_json::from_str::<Envelope>(message).is_err());
        }
    }
    #[test]
    fn untested_versions_and_guessed_locations_fail() {
        let mut h = Hello {
            version: 1,
            plugin_id: PLUGIN_ID.into(),
            plugin_version: PLUGIN_VERSION.into(),
            zotero_version: "10.0.7".into(),
            server_id: None,
        };
        assert!(h.validate().is_err());
        h.zotero_version = "10.0.6".into();
        h.validate().unwrap();
        assert!(
            !Target {
                page_index: Some(2000),
                annotation_key: None
            }
            .validate()
        );
        assert!(
            !Target {
                page_index: None,
                annotation_key: Some("../x".into())
            }
            .validate()
        );
    }
}
