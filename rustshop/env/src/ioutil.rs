use std::{
    io::{self, Write},
    path::Path,
};

use derive_more::Display;
use error_stack::{Context, Result, ResultExt};

#[derive(Debug, Display)]
pub enum CfgFileIOError {
    #[display(fmt = "IO Error")]
    Io,
    #[display(fmt = "Serialization error")]
    Serde,
    #[display(fmt = "Can't write to `/`")]
    RootPath,
}

pub type CfgFileIOResult<T> = Result<T, CfgFileIOError>;

impl Context for CfgFileIOError {}

pub fn save_to_yaml_file<T>(path: &Path, t: &T) -> CfgFileIOResult<()>
where
    T: ::serde::Serialize,
{
    std::fs::create_dir_all(path.parent().ok_or(CfgFileIOError::RootPath)?)
        .change_context(CfgFileIOError::Io)?;
    let text = serde_yaml::to_string(t).change_context(CfgFileIOError::Serde)?;
    store_str_to_file(path, &text).change_context(CfgFileIOError::Io)?;
    Ok(())
}

#[allow(unused)]
pub fn read_from_yaml_file<T>(path: &Path) -> Result<T, CfgFileIOError>
where
    T: ::serde::de::DeserializeOwned,
{
    let text = std::fs::read_to_string(path).change_context(CfgFileIOError::Io)?;

    Ok(serde_yaml::from_str(&text).change_context(CfgFileIOError::Serde)?)
}

#[inline]
pub fn store_str_to_file(path: &Path, s: &str) -> io::Result<()> {
    store_to_file_with(path, |f| f.write_all(s.as_bytes())).and_then(|res| res)
}

pub fn store_to_file_with<E, F>(path: &Path, f: F) -> io::Result<std::result::Result<(), E>>
where
    F: Fn(&mut dyn io::Write) -> std::result::Result<(), E>,
{
    std::fs::create_dir_all(path.parent().expect("Not a root path"))?;
    let tmp_path = path.with_extension("tmp");
    let mut file = std::fs::File::create(&tmp_path)?;
    if let Err(e) = f(&mut file) {
        return Ok(Err(e));
    }
    file.flush()?;
    file.sync_data()?;
    drop(file);
    std::fs::rename(tmp_path, path)?;
    Ok(Ok(()))
}
