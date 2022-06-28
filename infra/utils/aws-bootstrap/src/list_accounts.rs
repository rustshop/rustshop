use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "UPPERCASE"))]
pub enum Status {
    Active,
    #[serde(other)]
    Other,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct Account {
    pub id: String,
    pub arn: String,
    pub email: String,
    pub name: String,
    pub status: Status,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
#[allow(dead_code)]
pub struct Output {
    pub accounts: Vec<Account>,
}
