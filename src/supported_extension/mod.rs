mod supported_extension;

pub static CONFIG_EXTENSIONS: SupportedExtension = SupportedExtension::new(&["yml", "yaml"]);


pub use supported_extension::SupportedExtension;
