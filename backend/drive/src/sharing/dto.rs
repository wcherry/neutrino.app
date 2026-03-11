use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::sharing::model::ShareLinkRecord;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum LinkVisibility {
    /// Publicly visible (indexable)
    Public,
    /// Accessible to anyone who has the link
    AnyoneWithLink,
}

impl LinkVisibility {
    pub fn as_str(&self) -> &str {
        match self {
            LinkVisibility::Public => "public",
            LinkVisibility::AnyoneWithLink => "anyone_with_link",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "public" => Some(LinkVisibility::Public),
            "anyone_with_link" => Some(LinkVisibility::AnyoneWithLink),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum LinkRole {
    Viewer,
    Commenter,
    Editor,
}

impl LinkRole {
    pub fn as_str(&self) -> &str {
        match self {
            LinkRole::Viewer => "viewer",
            LinkRole::Commenter => "commenter",
            LinkRole::Editor => "editor",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "viewer" => Some(LinkRole::Viewer),
            "commenter" => Some(LinkRole::Commenter),
            "editor" => Some(LinkRole::Editor),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShareLinkResponse {
    pub id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub token: String,
    pub visibility: String,
    pub role: String,
    pub expires_at: Option<String>,
    pub is_active: bool,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ShareLinkRecord> for ShareLinkResponse {
    fn from(r: ShareLinkRecord) -> Self {
        ShareLinkResponse {
            id: r.id,
            resource_type: r.resource_type,
            resource_id: r.resource_id,
            token: r.token,
            visibility: r.visibility,
            role: r.role,
            expires_at: r.expires_at.map(|dt| dt.to_string()),
            is_active: r.is_active,
            created_by: r.created_by,
            created_at: r.created_at.to_string(),
            updated_at: r.updated_at.to_string(),
        }
    }
}

/// Request body for creating or replacing a share link.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpsertShareLinkRequest {
    #[serde(default = "default_visibility")]
    pub visibility: LinkVisibility,
    #[serde(default = "default_role")]
    pub role: LinkRole,
    /// ISO 8601 datetime string, e.g. "2026-12-31T23:59:59"
    pub expires_at: Option<String>,
}

fn default_visibility() -> LinkVisibility {
    LinkVisibility::AnyoneWithLink
}

fn default_role() -> LinkRole {
    LinkRole::Viewer
}

/// Request body for updating share link settings.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateShareLinkRequest {
    pub visibility: Option<LinkVisibility>,
    pub role: Option<LinkRole>,
    /// ISO 8601 datetime string. Pass null to remove expiration.
    pub expires_at: Option<Option<String>>,
    pub is_active: Option<bool>,
}

/// Response when resolving a share link token (public endpoint).
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedShareLinkResponse {
    pub resource_type: String,
    pub resource_id: String,
    pub role: String,
    pub visibility: String,
    pub expires_at: Option<String>,
    /// Basic resource info
    pub resource_name: String,
}
