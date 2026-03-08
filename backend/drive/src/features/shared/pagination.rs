use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderDirection {
    Asc,
    Desc,
}

/// Generic paginated list query usable by any feature.
/// `F` is the domain-specific order field enum (must implement `Deserialize`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery<F> {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub order_by: Option<F>,
    pub direction: Option<OrderDirection>,
}

fn default_limit() -> i64 {
    50
}
