pub mod flate;

#[cfg(not(target_arch = "wasm32"))]
use self::flate::{unzip, zip};

#[cfg(not(target_arch = "wasm32"))]
use anyhow::{anyhow, Context};

/// Settings for where the data should be stored.
///
/// - On desktop, data is stored to `/path/to/datadir/{bin_name}/v{version}/{profile}.dat`.
/// - On localstorage, data is stored under the key `"{bin_name}/v{version}/{profile}"`.
#[derive(Debug, Clone)]
pub struct Location {
    /// The name of your binary crate, via `env!("CARGO_PKG_NAME")`.
    ///
    /// You probably don't want to edit this, but you might if you want several of your games
    /// to talk to each other, for example.
    /// (Note this example probably won't be portable on the web because localstorage only stores things
    /// to one [origin](https://developer.mozilla.org/en-US/docs/Glossary/Origin). If you host all your games
    /// yourself on one domain it might be, but on itch.io for example it won't be.)
    pub bin_name: String,
    /// The version of your binary crate, via `env!("CARGO_PKG_VERSION")`.
    ///
    /// You might want to edit this if your serialization format updates less often than your
    /// crate version.
    pub version: String,
    /// The profile name.
    ///
    /// If you have multiple players, you definitely want to update this.
    ///
    /// Is `"default"` by default.
    pub profile: String,
}

impl Location {
    /// Get the path to save the data to.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn path(&self) -> anyhow::Result<std::path::PathBuf> {
        let root = dirs::data_dir().ok_or_else(|| anyhow!("Couldn't find data dir!"))?;
        Ok(root
            .join(&self.bin_name)
            .join(format!("v{}", &self.version))
            .join(&self.profile)
            .with_extension("dat"))
    }

    /// Get the key to save data to localstorage under
    #[cfg(target_arch = "wasm32")]
    pub fn key(&self) -> String {
        format!("{}/v{}/{}", &self.bin_name, &self.version, &self.profile)
    }
}

impl Default for Location {
    fn default() -> Self {
        Self {
            bin_name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            profile: String::from("default"),
        }
    }
}

/// Save some data to the default location.
///
/// If this returns `Err` it's *not* guaranteed that we made no edits
/// to the filesystem or localstorage.
pub fn save<T: AsRef<[u8]>>(data: T) -> anyhow::Result<()> {
    #[cfg(target_arch = "wasm32")]
    {
        wasm::save(&Location::default().key(), data.as_ref())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        save_to(data, &Location::default())
    }
}

/// Save some data to the given location.
///
/// If this returns `Err` it's *not* guaranteed that we made no edits
/// to the filesystem or localstorage.
pub fn save_to<T: AsRef<[u8]>>(data: T, location: &Location) -> anyhow::Result<()> {
    #[cfg(target_arch = "wasm32")]
    {
        wasm::save(&location.key(), data.as_ref())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = location.path()?;
        std::fs::create_dir_all(
            &path
                .parent()
                .ok_or_else(|| anyhow!("Couldn't get parent of {:?}", &path))?,
        )?;

        let data = zip(data)?;
        std::fs::write(&path, &data)
            .with_context(|| anyhow!("When writing to the file at {:?}", &path))?;
        Ok(())
    }
}

/// Load some data from the default location
pub fn load() -> anyhow::Result<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    {
        wasm::load(&Location::default().key())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        load_from(&Location::default())
    }
}

/// Load some data from the given location.
pub fn load_from(location: &Location) -> anyhow::Result<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    {
        wasm::load(&location.key())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let zipped = std::fs::read(location.path()?).context("When reading the file")?;
        unzip(&zipped)
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::flate::zip64;
    use crate::{js_convert::FromJsObject, storage::flate::unzip64};

    use anyhow::{anyhow, bail, Context};
    use sapp_jsutils::{JsObject, JsObjectWeak};

    extern "C" {
        /// returns `Result<(), String>`
        fn storage_save(key: JsObjectWeak, val: JsObjectWeak) -> JsObject;
        /// returns `Result<String, String>`
        fn storage_load(key: JsObjectWeak) -> JsObject;
    }

    pub fn save(key: &str, val: &[u8]) -> anyhow::Result<()> {
        let key = JsObject::string(&key);
        let val = JsObject::string(&zip64(val)?);

        let result = unsafe { storage_save(key.weak(), val.weak()) };
        let result = Result::<(), String>::from_js(result)
            .context("When trying to turn the returned value into a Result")?;
        match result {
            Ok(()) => Ok(()),
            Err(oh_no) => bail!(anyhow!(oh_no).context("When trying to save to localstorage")),
        }
    }
    pub fn load(key: &str) -> anyhow::Result<Vec<u8>> {
        let key = JsObject::string(&key);

        let result = unsafe { storage_load(key.weak()) };
        let result = Result::<String, String>::from_js(result)
            .context("When trying to turn the returned value into a Result")?;
        let data =
            result.map_err(|e| anyhow!(e).context("When trying to load from localstorage"))?;
        unzip64(&data)
    }
}
