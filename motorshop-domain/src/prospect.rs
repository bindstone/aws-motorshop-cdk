#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Prospect {
    pub name: String,
    pub model: String,
}