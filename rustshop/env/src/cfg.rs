use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::default::Default;

pub type AccountName = String;
pub type ClusterName = String;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ShopCfg {
    pub name: String,
    pub domain: String,
}

impl ShopCfg {
    pub fn new(name: String, domain: String) -> Self {
        Self { name, domain }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShopAccountCfg {
    /// Full account name used during CloudFormation bootstrap (e.g. `rustshop-prod`)
    pub bootstrap_name: String,
    /// Account suffix name during CloudFormation bootstrap
    pub bootstrap_aws_region: String,

    pub clusters: BTreeMap<ClusterName, ShopClusterCfg>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserAccountCfg {
    // TODO: remove alias in the future
    #[serde(alias = "profile")]
    pub aws_profile: String,

    #[serde(flatten)]
    pub clusters: BTreeMap<ClusterName, UserClusterCfg>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountCfg {
    pub shop: ShopAccountCfg,
    pub user: UserAccountCfg,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShopClusterCfg {
    pub domain: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserClusterCfg {
    // TODO: remove in the future
    #[serde(alias = "ctx")]
    pub kube_ctx: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClusterCfg {
    pub shop: ShopClusterCfg,
    pub user: UserClusterCfg,
}

// #[derive(Debug, Serialize, Deserialize, Default)]
// pub struct RegionCfg {
//     pub region: String,
//     pub az: String,
// }

// #[derive(Debug, Serialize, Deserialize, Default)]
// pub struct RegionCfgOpt {
//     pub region: Option<String>,
//     pub az: Option<String>,
// }

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UserYaml {
    pub accounts: BTreeMap<AccountName, UserAccountCfg>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShopYaml {
    #[serde(flatten)]
    pub shop: ShopCfg,
    pub accounts: BTreeMap<AccountName, ShopAccountCfg>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ContextYaml {
    pub account: Option<String>,
    pub cluster: Option<String>,
    pub namespace: Option<String>,
}
