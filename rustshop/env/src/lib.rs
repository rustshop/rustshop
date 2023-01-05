use derive_more::Display;
use error_stack::{bail, Context, IntoReport, Result, ResultExt};
use std::default::Default;
use std::io;
use std::ops::{Deref, DerefMut};
use std::{collections::BTreeMap, path::PathBuf};
use tracing::debug;

pub mod cfg;
pub use cfg::*;

mod ioutil;

#[derive(Display)]
pub struct Suggestion(&'static str);

#[derive(Debug, Display)]
pub enum EnvError {
    #[display(fmt = "Could not load rustshop environment")]
    Load,
    #[display(fmt = "Shop does not exist")]
    ShopDoesNotExist,
    #[display(fmt = "Could not load a file")]
    FileLoadFile,
    #[display(fmt = "File already exist: {}", "path.display()")]
    FileExists { path: PathBuf },
    #[display(fmt = "Update failed: {}", "path.display()")]
    FileUpdateFailed { path: PathBuf },

    #[display(fmt = "Account not set")]
    AccountNotSet,
    #[display(fmt = "Account exists: {}", name)]
    AccountExists { name: String },
    #[display(fmt = "Account does not exist: {}", name)]
    AccountDoesNotExist { name: String },
    #[display(
        fmt = "Account data inconsistent between shop and user config: {}",
        name
    )]
    InconsistentAccountData { name: String },

    #[display(fmt = "Account user data missing: {}", name)]
    AccountNotConfigured { name: String },

    #[display(fmt = "Cluster not set")]
    ClusterNotSet,
    #[display(fmt = "Cluster exists: {}", name)]
    ClusterExists { name: String },
    #[display(fmt = "Cluster does not exist: {}", name)]
    ClusterDoesNotExist { name: String },
    #[display(
        fmt = "Cluster data inconsistent between shop and user config: {}",
        name
    )]
    InconsistentClusterData { name: String },
    #[display(fmt = "Cluster user data missing: {}", name)]
    ClusterNotConfigured { name: String },
}

pub type EnvResult<T> = Result<T, EnvError>;

impl Context for EnvError {}

#[derive(Debug, Display)]
pub enum RootError {
    #[display(fmt = "Root directory not set")]
    NotSet,
    #[display(fmt = "Root directory does not exist")]
    DoesntExist,
}

impl Context for RootError {}

pub type RootResult<T> = Result<T, RootError>;

pub struct EnvRoot {
    path: PathBuf,
}

impl EnvRoot {
    pub const ROOT_SUBDIR: &'static str = ".rustshop";

    pub fn load_path() -> RootResult<PathBuf> {
        let name = load_env_var("RUSTSHOP_ROOT").change_context(RootError::NotSet)?;
        let path = PathBuf::from(name);

        if !path.exists() {
            bail!(RootError::DoesntExist);
        }

        Ok(path)
    }

    pub fn load() -> EnvResult<Self> {
        let path = Self::load_path().change_context(EnvError::Load)?;
        debug!(root = %path.display(), "Loading env root");

        Ok(Self { path })
    }

    pub fn add_shop(&self, name: String, domain: String) -> EnvResult<()> {
        debug!(name, domain, "Add shop");
        let shop = ShopCfg { name, domain };

        let shop_yaml = ShopYaml {
            shop,
            accounts: BTreeMap::new(),
        };

        if let Some(_shop_yaml) = self.load_shop_yaml_opt()? {
            // we don't allow overwritting shop file; this should be done once and once only
            bail!(EnvError::FileExists {
                path: self.shop_yaml_path()
            });
        }

        let user_yaml = if let Some(user_yaml) = self.load_user_yaml_opt()? {
            user_yaml
        } else {
            UserYaml::default()
        };

        self.write_shop_yaml(&shop_yaml)?;
        self.write_user_yaml(&user_yaml)?;
        Ok(())
    }

    pub fn root_cfg_dir(&self) -> PathBuf {
        self.path.join(Self::ROOT_SUBDIR)
    }

    pub fn shop_yaml_path(&self) -> PathBuf {
        self.path.join(Self::ROOT_SUBDIR).join("shop.yaml")
    }

    pub fn user_yaml_path(&self) -> PathBuf {
        self.path.join(Self::ROOT_SUBDIR).join("user.yaml")
    }

    pub fn context_yaml_path(&self) -> PathBuf {
        self.root_cfg_dir().join("state").join("context.yaml")
    }

    fn load_shop_yaml_opt(&self) -> EnvResult<Option<ShopYaml>> {
        let path = self.shop_yaml_path();
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(
            ioutil::read_from_yaml_file(&path).change_context(EnvError::FileLoadFile)?,
        ))
    }

    fn load_shop_yaml(&self) -> EnvResult<ShopYaml> {
        Ok(if let Some(shop_yaml) = self.load_shop_yaml_opt()? {
            shop_yaml
        } else {
            bail!(EnvError::ShopDoesNotExist);
        })
    }

    pub fn load_shop_cfg_opt(&self) -> EnvResult<Option<ShopCfg>> {
        Ok(self.load_shop_yaml_opt()?.map(|shop_yaml| shop_yaml.shop))
    }

    pub fn load_user_yaml_opt(&self) -> EnvResult<Option<UserYaml>> {
        let path = self.user_yaml_path();
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(
            ioutil::read_from_yaml_file(&path).change_context(EnvError::FileLoadFile)?,
        ))
    }

    pub fn load_user_yaml(&self) -> EnvResult<UserYaml> {
        Ok(if let Some(user_yaml) = self.load_user_yaml_opt()? {
            user_yaml
        } else {
            bail!(EnvError::ShopDoesNotExist);
        })
    }

    fn load_context_yaml_opt(&self) -> EnvResult<Option<ContextYaml>> {
        let path = self.context_yaml_path();
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(
            ioutil::read_from_yaml_file(&path).change_context(EnvError::FileLoadFile)?,
        ))
    }

    fn write_shop_yaml(&self, new_shop_yaml: &ShopYaml) -> EnvResult<()> {
        ioutil::save_to_yaml_file(&self.shop_yaml_path(), new_shop_yaml).change_context_lazy(|| {
            EnvError::FileUpdateFailed {
                path: self.shop_yaml_path(),
            }
        })
    }
    fn write_user_yaml(&self, new_user_yaml: &UserYaml) -> EnvResult<()> {
        ioutil::save_to_yaml_file(&self.user_yaml_path(), new_user_yaml).change_context_lazy(|| {
            EnvError::FileUpdateFailed {
                path: self.user_yaml_path(),
            }
        })
    }

    fn write_context_yaml(&self, context_yaml: &ContextYaml) -> EnvResult<()> {
        ioutil::save_to_yaml_file(&self.context_yaml_path(), context_yaml).change_context_lazy(
            || EnvError::FileUpdateFailed {
                path: self.context_yaml_path(),
            },
        )
    }
}

impl Deref for Env {
    type Target = EnvRoot;

    fn deref(&self) -> &Self::Target {
        &self.root
    }
}

impl DerefMut for Env {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.root
    }
}
pub struct Env {
    root: EnvRoot,
    shop: ShopYaml,
    shop_dirty: bool,
    user: UserYaml,
    user_dirty: bool,
    context_path: ContextYaml,
    context_dirty: bool,
}

impl Env {
    pub const NO_BIN_WRAP_ENV_NAME: &'static str = "RUSTSHOP_NO_BIN_WRAP";

    pub fn load() -> EnvResult<Self> {
        let root = EnvRoot::load()?;

        Ok(Self {
            shop: root.load_shop_yaml()?,
            // if the user config isn't there, just start with an empty one
            // instead of erroring out
            user: root.load_user_yaml_opt()?.unwrap_or_default(),
            context_path: root
                .load_context_yaml_opt()?
                .unwrap_or_else(|| ContextYaml::default()),

            shop_dirty: false,
            user_dirty: false,
            context_dirty: false,
            root,
        })
    }

    // technically it doesn't require `&mut`, but it
    // will prevent some mistakes
    pub fn write(&mut self) -> EnvResult<()> {
        if self.shop_dirty {
            debug!("Saving shop.yaml");
            self.root.write_shop_yaml(&self.shop)?;
            self.shop_dirty = false;
        }
        if self.user_dirty {
            debug!("Saving user.yaml");
            self.root.write_user_yaml(&self.user)?;
            self.user_dirty = false;
        }

        if self.context_dirty {
            debug!("Saving context.yaml");
            self.root.write_context_yaml(&self.context_path)?;
            self.context_dirty = false;
        }
        Ok(())
    }

    pub fn get_shop_ref(&self) -> &ShopCfg {
        &self.shop.shop
    }

    pub fn get_shop_mut(&mut self) -> &mut ShopCfg {
        &mut self.shop.shop
    }

    pub fn get_shop_account_mut_opt<'env, 'name>(
        &'env mut self,
        name: &'name str,
    ) -> Option<&'env mut ShopAccountCfg> {
        self.shop_dirty = true;
        self.shop.accounts.get_mut(name)
    }

    pub fn get_shop_account_mut<'env, 'name>(
        &'env mut self,
        name: &'name str,
    ) -> EnvResult<&'env mut ShopAccountCfg> {
        self.get_shop_account_mut_opt(name)
            .ok_or(EnvError::AccountNotConfigured {
                name: name.to_owned(),
            })
            .into_report()
    }

    pub fn get_shop_account_ref_opt<'env, 'name>(
        &'env self,
        name: &'name str,
    ) -> Option<&'env ShopAccountCfg> {
        self.shop.accounts.get(name)
    }

    pub fn get_shop_account_ref<'env, 'name>(
        &'env self,
        name: &'name str,
    ) -> EnvResult<&'env ShopAccountCfg> {
        self.get_shop_account_ref_opt(name)
            .ok_or(EnvError::AccountNotConfigured {
                name: name.to_owned(),
            })
            .into_report()
    }

    pub fn get_account_ref_opt<'env, 'name>(
        &'env self,
        name: &'name str,
    ) -> EnvResult<Option<EnvAccountRef<'env>>> {
        Ok(
            match (self.shop.accounts.get(name), self.user.accounts.get(name)) {
                (Some(shop_account_cfg), Some(user_account_cfg)) => Some(EnvAccountRef {
                    shop: shop_account_cfg,
                    user: user_account_cfg,
                }),
                (None, None) => None,
                (None, Some(_)) => {
                    bail!(EnvError::InconsistentAccountData {
                        name: name.to_owned()
                    })
                }
                (Some(_), None) => Err(EnvError::AccountNotConfigured {
                    name: name.to_owned(),
                })
                .into_report()
                .attach(Suggestion("Use `rustshop configure account`"))?,
            },
        )
    }

    pub fn get_account_mut_opt<'env, 'name>(
        &'env mut self,
        name: &'name str,
    ) -> EnvResult<Option<EnvAccountMut<'env>>> {
        Ok(
            match (
                self.shop.accounts.get_mut(name),
                self.user.accounts.get_mut(name),
            ) {
                (Some(shop_account_cfg), Some(user_account_cfg)) => {
                    self.shop_dirty = true;
                    self.user_dirty = true;
                    Some(EnvAccountMut {
                        shop: shop_account_cfg,
                        user: user_account_cfg,
                    })
                }
                (None, None) => None,
                (None, Some(_)) => {
                    bail!(EnvError::InconsistentAccountData {
                        name: name.to_owned()
                    })
                }
                (Some(_), None) => bail!(EnvError::AccountNotConfigured {
                    name: name.to_owned()
                }),
            },
        )
    }

    pub fn get_account_ref<'env, 'name>(
        &'env self,
        name: &'name str,
    ) -> EnvResult<EnvAccountRef<'env>> {
        Ok(self
            .get_account_ref_opt(name)?
            .ok_or_else(|| EnvError::AccountDoesNotExist {
                name: name.to_owned(),
            })?)
    }

    pub fn get_account_mut<'env, 'name>(
        &'env mut self,
        name: &'name str,
    ) -> EnvResult<EnvAccountMut<'env>> {
        Ok(self
            .get_account_mut_opt(name)?
            .ok_or_else(|| EnvError::AccountDoesNotExist {
                name: name.to_owned(),
            })?)
    }

    pub fn add_account(&mut self, name: &str, aws_region: &str) -> EnvResult<ShopAccountCfg> {
        debug!(name, "Add account");

        if let Some(_account_cfg) = self.get_account_mut_opt(name)? {
            bail!(EnvError::AccountExists {
                name: name.to_owned()
            });
        }

        let shop_cfg = ShopAccountCfg {
            bootstrap_name: format!("{}-{}", self.shop.shop.name, name),
            bootstrap_aws_region: aws_region.to_string(),
            clusters: BTreeMap::new(),
        };
        self.shop.accounts.insert(name.to_owned(), shop_cfg.clone());
        self.shop_dirty = true;
        self.write()?;
        Ok(shop_cfg)
    }

    pub fn add_cluster(
        &mut self,
        account_name: &str,
        cluster_name: &str,
    ) -> EnvResult<ShopClusterCfg> {
        debug!(
            account = account_name,
            cluster = cluster_name,
            "Add cluster"
        );

        let shop_domain = self.shop.shop.domain.clone();

        let account_cfg = self.get_shop_account_mut(&account_name)?;

        if let Some(_cluster_ref) = account_cfg.clusters.get(cluster_name) {
            bail!(EnvError::ClusterExists {
                name: cluster_name.to_owned()
            });
        }

        let shop_cluster = ShopClusterCfg {
            domain: format!("{}.k8s.{}", cluster_name, shop_domain),
        };

        account_cfg
            .clusters
            .entry(cluster_name.to_owned())
            .or_insert(shop_cluster.clone());

        self.write()?;

        Ok(shop_cluster)
    }

    pub fn configure_account(&mut self, name: &str, profile: &str) -> EnvResult<AccountCfg> {
        if !self.shop.accounts.contains_key(name) {
            bail!(EnvError::AccountDoesNotExist {
                name: name.to_owned()
            });
        }
        self.user
            .accounts
            .entry(name.to_owned())
            .or_insert(UserAccountCfg {
                aws_profile: profile.to_owned(),
                clusters: Default::default(),
            })
            .aws_profile = profile.to_owned();

        self.user_dirty = true;
        self.write()?;

        Ok(self
            .get_account_mut_opt(name)?
            .expect("Account must be there")
            .into())
    }

    pub fn configure_cluster(
        &mut self,
        account_name: Option<&str>,
        name: &str,
        ctx: &str,
    ) -> EnvResult<ClusterCfg> {
        let account_name = if let Some(account_name) = account_name {
            account_name.to_owned()
        } else {
            self.get_context()?
                .account
                .ok_or_else(|| EnvError::AccountNotSet)?
                .0
                .to_owned()
        };

        let mut account_cfg = self.get_account_mut(&account_name)?;

        let cluster_cfg = account_cfg.configure_cluster(name, ctx)?;

        self.write()?;

        Ok(cluster_cfg)
    }

    /// Narrow down context to elements that actually exist
    /// Widen up context when only one sub-element is availble
    pub fn normalize_context_path(
        &self,
        context_path: Option<ContextYaml>,
        widen: bool,
    ) -> EnvResult<ContextYaml> {
        // no context is the same as context with everyting empty
        let context_path = context_path.unwrap_or(ContextYaml::default());

        let account_opt = if let Some(account) = context_path.account {
            if let Some(account_ref) = self.shop.accounts.get(&account) {
                Some((account.to_owned(), account_ref))
            } else {
                None
            }
        } else {
            None
        };

        let account = if let Some(account) = account_opt {
            account
        } else if widen && self.shop.accounts.len() == 1 {
            let only_account = self
                .shop
                .accounts
                .iter()
                .next()
                .expect("at least onc account");

            (only_account.0.to_owned(), only_account.1)
        } else {
            return Ok(ContextYaml {
                account: None,
                cluster: None,
                namespace: None,
            });
        };

        let cluster = if let Some(cluster) = context_path
            .cluster
            .and_then(|cluster| account.1.clusters.get(&cluster).map(|c| (cluster, c)))
        {
            (cluster.0.to_owned(), cluster.1)
        } else if widen && account.1.clusters.len() == 1 {
            let only_cluster = account
                .1
                .clusters
                .iter()
                .next()
                .expect("at least one cluster");
            (only_cluster.0.to_owned(), only_cluster.1)
        } else {
            return Ok(ContextYaml {
                account: Some(account.0),
                cluster: None,
                namespace: None,
            });
        };

        Ok(ContextYaml {
            account: Some(account.0),
            cluster: Some(cluster.0),
            namespace: context_path.namespace,
        })
    }

    pub fn resolve_context_path(&self, context_path: &ContextYaml) -> EnvResult<EnvContext> {
        let account = if let Some(account) = &context_path.account {
            if let Some(account_ref) = self.get_account_ref_opt(&account)? {
                (account, account_ref)
            } else {
                return Ok(EnvContext::default());
            }
        } else {
            return Ok(EnvContext::default());
        };

        let cluster = if let Some(cluster) = &context_path.cluster {
            if let Some(cluster_ref) = account.1.get_cluster_ref_opt(&cluster)? {
                (cluster, cluster_ref)
            } else {
                return Ok(EnvContext {
                    account: Some((account.0.to_owned(), account.1.into())),
                    ..EnvContext::default()
                });
            }
        } else {
            return Ok(EnvContext {
                account: Some((account.0.to_owned(), account.1.into())),
                ..EnvContext::default()
            });
        };

        Ok(EnvContext {
            account: Some((account.0.to_owned(), account.1.into())),
            cluster: Some((cluster.0.to_owned(), cluster.1.into())),
            namespace: context_path.namespace.to_owned(),
        })
    }

    pub fn switch_account(&mut self, name: &str) -> EnvResult<EnvContext> {
        let context_path = self.normalize_context_path(self.load_context_yaml_opt()?, true)?;

        let context = self.resolve_context_path(&ContextYaml {
            account: Some(name.to_owned()),
            ..context_path
        })?;
        self.context_path = context.clone().into();
        self.context_dirty = true;

        self.write()?;

        Ok(context)
    }

    pub fn switch_cluster(&mut self, name: &str) -> EnvResult<EnvContext> {
        let context_path = self.normalize_context_path(self.load_context_yaml_opt()?, true)?;

        let context = self.resolve_context_path(&ContextYaml {
            cluster: Some(name.to_owned()),
            ..context_path
        })?;

        self.context_path = context.clone().into();
        self.context_dirty = true;

        self.write()?;

        Ok(context)
    }

    pub fn switch_namespace(&mut self, name: &str) -> EnvResult<EnvContext> {
        let context_path = self.normalize_context_path(self.load_context_yaml_opt()?, true)?;

        let context = self.resolve_context_path(&ContextYaml {
            namespace: Some(name.to_owned()),
            ..context_path
        })?;

        self.context_path = context.clone().into();
        self.context_dirty = true;

        self.write()?;

        Ok(context)
    }

    pub fn get_context(&self) -> EnvResult<EnvContext> {
        let context_path = self.normalize_context_path(self.load_context_yaml_opt()?, true)?;

        self.resolve_context_path(&context_path)
    }

    /// Like `get_context`, but will error out if account not set
    pub fn get_context_account(&self) -> EnvResult<EnvContext> {
        let context = self.get_context()?;

        if context.account.is_none() {
            bail!(EnvError::AccountNotSet);
        }

        Ok(context)
    }

    /// Like `get_context`, but will error out if account not set
    pub fn get_context_cluster(&self) -> EnvResult<EnvContext> {
        let context = self.get_context()?;

        if context.cluster.is_none() {
            bail!(EnvError::ClusterNotSet);
        }

        Ok(context)
    }

    pub fn write_ctx_info_to<W>(
        &self,
        context: EnvContext,
        w: &mut W,
    ) -> std::result::Result<(), io::Error>
    where
        W: io::Write,
    {
        write!(
            w,
            "Context: shop={} ({})",
            self.get_shop_ref().name,
            self.get_shop_ref().domain
        )?;

        if let Some(account) = context.account {
            write!(
                w,
                "; account={} ({})",
                account.0, account.1.user.aws_profile
            )?;
            if let Some(cluster) = context.cluster {
                write!(w, "; cluster={} ({})", cluster.0, cluster.1.user.kube_ctx)?;
                if let Some(namespace) = context.namespace {
                    write!(w, "; namespace={}", namespace)?;
                }
            }
        }

        writeln!(w, "")?;

        Ok(())
    }

    pub fn shop_cfg(&self) -> &ShopCfg {
        &self.shop.shop
    }
}

impl<'env> From<EnvAccountMut<'env>> for AccountCfg {
    fn from(acc: EnvAccountMut) -> Self {
        Self {
            shop: acc.shop.to_owned(),
            user: acc.user.to_owned(),
        }
    }
}

impl<'env> From<EnvClusterMut<'env>> for ClusterCfg {
    fn from(cluster: EnvClusterMut) -> Self {
        Self {
            shop: cluster.shop.to_owned(),
            user: cluster.user.to_owned(),
        }
    }
}

impl<'env> From<EnvAccountRef<'env>> for AccountCfg {
    fn from(acc: EnvAccountRef) -> Self {
        Self {
            shop: acc.shop.to_owned(),
            user: acc.user.to_owned(),
        }
    }
}

impl<'env> From<EnvClusterRef<'env>> for ClusterCfg {
    fn from(cluster: EnvClusterRef) -> Self {
        Self {
            shop: cluster.shop.to_owned(),
            user: cluster.user.to_owned(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct EnvContext {
    pub account: Option<(String, AccountCfg)>,
    pub cluster: Option<(String, ClusterCfg)>,
    pub namespace: Option<String>,
}

impl Into<ContextYaml> for EnvContext {
    fn into(self) -> ContextYaml {
        ContextYaml {
            account: self.account.map(|account| account.0),
            cluster: self.cluster.map(|cluster| cluster.0),
            namespace: self.namespace,
        }
    }
}

#[derive(Debug, Display)]
pub enum EnvVarError {
    #[display(fmt = "Environment variable {} is not set", name)]
    Missing { name: String },
    #[display(fmt = "Environment variable {} is empty", name)]
    Empty { name: String },
}

pub type EnvVarResult<T> = Result<T, EnvVarError>;

impl Context for EnvVarError {}

pub fn load_env_var_opt(name: &str) -> EnvVarResult<Option<String>> {
    if let Some(val) = std::env::var(name).ok() {
        if val.is_empty() {
            bail!(EnvVarError::Empty {
                name: name.to_owned(),
            });
        } else {
            Ok(Some(val))
        }
    } else {
        Ok(None)
    }
}

pub fn load_env_var(name: &str) -> EnvVarResult<String> {
    let value = std::env::var(name).map_err(|_| EnvVarError::Missing {
        name: name.to_owned(),
    })?;

    if value.is_empty() {
        bail!(EnvVarError::Empty {
            name: name.to_owned(),
        });
    }

    Ok(value)
}

#[derive(Copy, Clone, Debug)]
pub struct EnvAccountRef<'env> {
    pub shop: &'env ShopAccountCfg,
    pub user: &'env UserAccountCfg,
}

impl<'env> EnvAccountRef<'env> {
    pub fn get_cluster_ref_opt(&self, name: &str) -> EnvResult<Option<EnvClusterRef>> {
        Ok(
            match (self.shop.clusters.get(name), self.user.clusters.get(name)) {
                (Some(shop), Some(user)) => Some(EnvClusterRef { shop, user }),
                (None, None) => None,
                (None, Some(_)) => {
                    bail!(EnvError::InconsistentClusterData {
                        name: name.to_owned()
                    })
                }
                (Some(_), None) => None,
            },
        )
    }
    pub fn get_cluster_ref(&self, name: &str) -> EnvResult<EnvClusterRef> {
        Ok(self
            .get_cluster_ref_opt(name)?
            .ok_or_else(|| EnvError::ClusterDoesNotExist {
                name: name.to_owned(),
            })?)
    }
}

pub struct EnvAccountMut<'env> {
    pub shop: &'env mut ShopAccountCfg,
    pub user: &'env mut UserAccountCfg,
}

impl<'env> EnvAccountMut<'env> {
    pub fn get_cluster_mut(&mut self, name: &str) -> EnvResult<Option<EnvClusterMut>> {
        Ok(
            match (
                self.shop.clusters.get_mut(name),
                self.user.clusters.get_mut(name),
            ) {
                (Some(shop), Some(user)) => Some(EnvClusterMut { shop, user }),
                (None, None) => None,
                (None, Some(_)) => {
                    bail!(EnvError::InconsistentClusterData {
                        name: name.to_owned()
                    })
                }
                (Some(_), None) => bail!(EnvError::ClusterNotConfigured {
                    name: name.to_owned()
                }),
            },
        )
    }

    pub fn get_cluster_ref_opt(&self, name: &str) -> EnvResult<Option<EnvClusterRef>> {
        Ok(
            match (self.shop.clusters.get(name), self.user.clusters.get(name)) {
                (Some(shop), Some(user)) => Some(EnvClusterRef { shop, user }),
                (None, None) => None,
                (None, Some(_)) => {
                    bail!(EnvError::InconsistentClusterData {
                        name: name.to_owned()
                    })
                }
                (Some(_), None) => None,
            },
        )
    }
    pub fn get_cluster_ref(&self, name: &str) -> EnvResult<EnvClusterRef> {
        Ok(self
            .get_cluster_ref_opt(name)?
            .ok_or_else(|| EnvError::ClusterDoesNotExist {
                name: name.to_owned(),
            })?)
    }

    pub fn configure_cluster(&mut self, name: &str, kube_ctx: &str) -> EnvResult<ClusterCfg> {
        if !self.shop.clusters.contains_key(name) {
            bail!(EnvError::ClusterDoesNotExist {
                name: name.to_owned()
            });
        }

        self.user
            .clusters
            .entry(name.to_owned())
            .or_insert(UserClusterCfg {
                kube_ctx: kube_ctx.to_owned(),
            })
            .kube_ctx = kube_ctx.to_owned();

        Ok(ClusterCfg {
            shop: self
                .shop
                .clusters
                .get(name)
                .expect("Cluster must exist")
                .clone(),
            user: self
                .user
                .clusters
                .get(name)
                .expect("Cluster must exist")
                .clone(),
        })
    }
}

pub struct EnvClusterRef<'env> {
    pub shop: &'env ShopClusterCfg,
    pub user: &'env UserClusterCfg,
}

pub struct EnvClusterMut<'env> {
    pub shop: &'env mut ShopClusterCfg,
    pub user: &'env mut UserClusterCfg,
}
/*
#[derive(Debug)]
/// Per account envs
pub struct AccountEnvs {
    pub aws_region: Option<String>,
    pub aws_profile: String,
    pub aws_account_id: String,
}

impl AccountEnvs {
    fn load(account_suffix: &str) -> Result<Self> {
        let account_suffix = account_suffix.to_uppercase();
        Ok(Self {
            aws_region: load_env_var_opt(&format!("RUSTSHOP_{}_AWS_REGION", account_suffix))?,
            aws_profile: load_env_var(&format!("RUSTSHOP_{}_AWS_PROFILE", account_suffix))?,
            aws_account_id: load_env_var(&format!("RUSTSHOP_{}_AWS_ACCOUNT_ID", account_suffix))?,
        })
    }

}

// #[derive(Debug)]
// pub struct Env {
//     pub shop_name: String,
//     pub domain: String,
//     pub account_suffix: String,
//     pub full_account_name: String,
//     pub account: AccountEnvs,
// }

impl Env {
    pub fn new_detect_no_profile_validation() -> Result<Self> {
        let shop_name = load_env_var("RUSTSHOP_NAME")?;
        let domain = load_env_var("RUSTSHOP_DOMAIN")?;
        let account_suffix = Self::load_account_suffix()?;

        let default_region = load_env_var_opt("RUSTSHOP_DEFAULT_AWS_REGION")?;

        let mut account = AccountEnvs::load(&account_suffix)?;

        account.aws_region = account.aws_region.or(default_region);

        let env = Env {
            full_account_name: format!("{shop_name}-{account_suffix}"),
            shop_name,
            account_suffix,
            account,
            domain,
        };
        trace!(account = format!("{env:?}"), "Env state");
        info!(
            account = env.account_suffix,
            aws_profile = env.account.aws_profile,
            "Env loaded"
        );

        Ok(env)
    }

    pub fn new_detect() -> Result<Self> {
        let env = Self::new_detect_no_profile_validation()?;
        env.check_profile_exists()?;
        Ok(env)
    }

    fn load_account_suffix() -> Result<String> {
        let root_path = load_env_var("RUSTSHOP_ROOT")?;
        let cwd = std::env::current_dir().map_err(CwdError::Io)?;

        let rel_path = cwd.strip_prefix(root_path).map_err(|_| CwdError::CwdRoot)?;

        Ok(rel_path
            .iter()
            .next()
            .ok_or_else(|| CwdError::CwdRoot)?
            .to_str()
            .ok_or_else(|| CwdError::CwdUnicode)?
            .to_owned())
    }

    /// Set the variables like for `aws` CLI, but prefixed with `TF_VAR_` so they
    /// are visible as Terraform variables.
    pub fn set_tf_aws_envs_on<'cmd>(&self, cmd: &'cmd mut Command) -> Result<&'cmd mut Command> {
        debug!(TF_VAR_SHOPNAME = self.shop_name, "Setting");
        cmd.env("TF_VAR_SHOPNAME", &self.shop_name);

        debug!(TF_VAR_ACCOUNT_SUFFIX = self.account_suffix, "Setting");
        cmd.env("TF_VAR_ACCOUNT_SUFFIX", &self.account_suffix);

        debug!(
            TF_VAR_AWS_ACCOUNT_ID = self.account.aws_account_id,
            "Setting"
        );
        cmd.env("TF_VAR_AWS_ACCOUNT_ID", &self.account.aws_account_id);

        debug!(TF_VAR_AWS_PROFILE = self.account.aws_profile, "Setting");
        cmd.env("TF_VAR_AWS_PROFILE", &self.account.aws_profile);

        if let Some(aws_region) = self.account.aws_region.as_ref() {
            debug!(TF_VAR_AWS_REGION = aws_region, "Setting");
            cmd.env("TF_VAR_AWS_REGION", aws_region);
        }
        Ok(cmd)
    }

    pub fn set_kops_envs_on<'cmd>(&self, cmd: &'cmd mut Command) -> Result<&'cmd mut Command> {
        let kops_state_store = self.get_kops_state_store_url();
        let kops_cluster_name = format!("{}.k8s.{}", self.account_suffix, self.domain);

        debug!(KOPS_STATE_STORE = kops_state_store, "Setting");
        cmd.env("KOPS_STATE_STORE", &kops_state_store);

        debug!(KOPS_CLUSTER_NAME = kops_cluster_name, "Setting");
        cmd.env("KOPS_CLUSTER_NAME", &kops_cluster_name);

        Ok(cmd)
    }

    pub fn get_kops_state_store_url(&self) -> String {
        format!("s3://{}-bootstrap-kops-state", self.full_account_name)
    }

    pub(crate) fn check_profile_exists(&self) -> Result<()> {
        let mut cmd = Command::new("aws");
        cmd.env(Env::NO_WRAP, "true");
        cmd.args(&["configure", "get", "name"]);
        self.account.set_aws_envs_on(&mut cmd);
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        let output = cmd.output()?;

        if output.status.code().unwrap_or(-1) == 255 {
            eprintln!("AWS Profile does not exist!");
            eprintln!("Consider creating it with:");
            eprintln!(
                "aws configure --profile {} set source_profile {}-root",
                self.account.aws_profile, self.shop_name
            );
            eprintln!(
                "aws configure --profile {} set role_arn 'arn:aws:iam::{}:role/OrganizationAccountAccessRole'",
                self.account.aws_profile,
                self.account.aws_account_id,
            );
            return Err(Error::ProfileDoesNotExist);
        }

        Ok(())
    }
}

*/
