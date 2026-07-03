use crate::asset_loader::AssetLoader;
use crate::challenges::{Package, PackageMetadata};
use crate::error::KonnektorenError;
use std::collections::HashMap;
use std::io::{Cursor, Read};
use zip::ZipArchive;

pub struct PackageReader;

impl PackageReader {
    pub async fn download(url: &str) -> Result<Vec<u8>, KonnektorenError> {
        let loader = AssetLoader::default();
        loader.load_binary(url).await
    }

    pub fn read(package_data: &[u8]) -> Result<Package, String> {
        let reader = Cursor::new(package_data);
        let mut archive =
            ZipArchive::new(reader).map_err(|e| format!("Failed to create ZIP archive: {}", e))?;

        let mut files = HashMap::new();
        let mut config = None;
        let mut custom = None;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read file from archive: {}", e))?;
            let file_name = file.name().to_string();

            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|e| format!("Failed to read file contents: {}", e))?;

            match file_name.as_str() {
                "config.yml" => {
                    config = Some(
                        serde_yaml::from_slice(&contents)
                            .map_err(|e| format!("Failed to parse config.yml: {}", e))?,
                    );
                }
                "challenge.yml" => {
                    custom = Some(
                        serde_yaml::from_slice(&contents)
                            .map_err(|e| format!("Failed to parse challenge.yml: {}", e))?,
                    );
                }
                _ => {
                    files.insert(file_name, contents);
                }
            }
        }

        let config = config.ok_or_else(|| "Missing config.yml in package".to_string())?;
        let custom = custom.unwrap_or_default();
        let metadata = PackageMetadata { config, custom };

        Ok(Package { metadata, files })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "ssr")]
    use std::env;

    #[test]
    fn test_package_reader() {
        let _ = tracing_subscriber::fmt::try_init();
        let package_data = include_bytes!("../../../../assets/articles-pkg.zip");
        let package = PackageReader::read(package_data).unwrap();

        assert_eq!(package.files.len(), 5);
        tracing::debug!("files {:?}", package.files.keys());
        assert!(package.get_html_file().is_some());
        assert!(package.get_css_file().is_some());
        assert!(package.get_js_file().is_some());
        assert!(package.get_results_file().is_some());
    }

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn test_package_reader_ssr() {
        use std::fs;
        use tempfile::TempDir;

        // 1. Create a temporary directory for BUILD_DIR
        let temp_dir = TempDir::new().unwrap();
        let build_dir_path = temp_dir.path().to_str().unwrap().to_string();

        // 2. Create a test zip file within the temporary directory
        let test_zip_content = include_bytes!("../../../assets/articles-pkg.zip");
        let test_zip_path = format!("{}/test_package.zip", build_dir_path);
        fs::write(&test_zip_path, test_zip_content).unwrap();

        // 3. Set the BUILD_DIR environment variable
        let _guard = SetEnvVariableGuard::new("BUILD_DIR", Some(build_dir_path.clone()));

        // 4. Instead of using the file:// protocol, read the file directly
        // since we're testing the read functionality more than the download part
        let package_data = fs::read(&test_zip_path).unwrap();
        let package = PackageReader::read(&package_data).unwrap();

        // 5. Verify the package contents
        assert_eq!(package.files.len(), 5);
        assert!(package.get_html_file().is_some());
        assert!(package.get_css_file().is_some());
        assert!(package.get_js_file().is_some());
        assert!(package.get_results_file().is_some());
    }

    #[cfg(feature = "ssr")]
    struct SetEnvVariableGuard {
        name: String,
        original_value: Option<String>,
    }

    #[cfg(feature = "ssr")]
    impl SetEnvVariableGuard {
        fn new(name: impl Into<String>, value: Option<String>) -> Self {
            let name = name.into();
            let original_value = env::var(&name).ok();

            match &value {
                Some(value) => unsafe { env::set_var(&name, value) },
                None => unsafe { env::remove_var(&name) },
            }

            Self {
                name,
                original_value,
            }
        }
    }

    #[cfg(feature = "ssr")]
    impl Drop for SetEnvVariableGuard {
        fn drop(&mut self) {
            match &self.original_value {
                Some(value) => unsafe { env::set_var(&self.name, value) },
                None => unsafe { env::remove_var(&self.name) },
            }
        }
    }
}
