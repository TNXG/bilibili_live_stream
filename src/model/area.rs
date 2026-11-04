use super::ids::deserialize_id;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Area {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AreaGroup {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: u64,
    pub name: String,
    pub list: Vec<Area>,
}
