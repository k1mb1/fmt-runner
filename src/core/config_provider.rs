use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait ConfigProvider: Serialize + DeserializeOwned + Default {}
impl<T> ConfigProvider for T where T: Serialize + DeserializeOwned + Default {}
