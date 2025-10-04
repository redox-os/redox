use std::{collections::HashMap, fs, sync::OnceLock};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct CookbookConfig {
    pub mirrors: HashMap<String, String>,
}

static CONFIG: OnceLock<CookbookConfig> = OnceLock::new();

pub fn init_config() {
    let config: CookbookConfig = {
        let toml_content = fs::read_to_string("cookbook.toml").unwrap_or("".to_owned());
        toml::from_str(&toml_content).unwrap_or(CookbookConfig::default())
    };

    CONFIG.set(config).expect("config is initialized twice");
}

pub fn translate_mirror(original_url: &str) -> String {
    let config = CONFIG.get().expect("Configuration is not initialized");

    let stripped_url = original_url
        .strip_prefix("https://")
        .or_else(|| original_url.strip_prefix("http://"))
        .unwrap_or(original_url);

    let mut best_match_prefix: Option<&String> = None;

    for prefix in config.mirrors.keys() {
        if stripped_url.starts_with(prefix) {
            match best_match_prefix {
                Some(current_best) if prefix.len() > current_best.len() => {
                    best_match_prefix = Some(prefix);
                }
                None => {
                    best_match_prefix = Some(prefix);
                }
                _ => {}
            }
        }
    }

    if let Some(prefix) = best_match_prefix {
        let mirror_base = config.mirrors.get(prefix).unwrap();
        let suffix = &stripped_url[prefix.len()..];
        let ptotocol = &original_url[..(original_url.len() - stripped_url.len())];
        return format!("{}{}{}", ptotocol, mirror_base, suffix);
    }

    original_url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_config() {
        let mut mirrors = HashMap::new();
        mirrors.insert("ftp.gnu.com".to_string(), "example.com/gnu".to_string());
        mirrors.insert(
            "github.com/foo/bar".to_string(),
            "github.com/baz/bar".to_string(),
        );
        mirrors.insert("github.com/a".to_string(), "github.com/b".to_string());

        let app_config = CookbookConfig { mirrors };

        // This will be called for each test. If the config is already set,
        // it will do nothing, which is fine as all tests use the same config.
        let _ = CONFIG.set(app_config);
    }

    #[test]
    fn test_exact_match() {
        setup_test_config();
        assert_eq!(translate_mirror("ftp.gnu.com"), "example.com/gnu");
        assert_eq!(translate_mirror("github.com/foo/bar"), "github.com/baz/bar");
    }

    #[test]
    fn test_prefix_match() {
        setup_test_config();
        assert_eq!(
            translate_mirror("https://github.com/a/c"),
            "https://github.com/b/c"
        );
        assert_eq!(
            translate_mirror("https://ftp.gnu.com/path/to/file"),
            "https://example.com/gnu/path/to/file"
        );
    }

    #[test]
    fn test_longest_prefix_match() {
        setup_test_config();
        // "github.com/foo/bar" is a longer and more specific prefix than "github.com/a",
        // so it should be chosen for the translation.
        assert_eq!(
            translate_mirror("https://github.com/foo/bar/baz"),
            "https://github.com/baz/bar/baz"
        );
    }

    #[test]
    fn test_no_match() {
        setup_test_config();
        assert_eq!(translate_mirror("www.rust-lang.org"), "www.rust-lang.org");
        assert_eq!(
            translate_mirror("http://github.com/unrelated/repo"),
            "http://github.com/unrelated/repo"
        );
    }
}
