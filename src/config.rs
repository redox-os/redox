use std::{collections::HashMap, env, fs, str::FromStr, sync::OnceLock};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct CookConfigOpt {
    /// whether to run offline
    pub offline: Option<bool>,
    /// whether to set jobs number instead of from nproc
    pub jobs: Option<usize>,
    /// whether to use TUI to allow parallel build
    /// default value is yes if "CI" env unset and STDIN is open.
    pub tui: Option<bool>,
    /// whether to write logs to build/logs dir, default true on TUI
    pub logs: Option<bool>,
    /// whether to ignore build errors
    pub nonstop: Option<bool>,
    /// whether to print verbose logs to certain commands
    /// build failure still be printed anyway
    pub verbose: Option<bool>,
    /// whether to always clean the build directory before building
    pub clean_build: Option<bool>,
    /// whether to always clean the target directory after building
    /// (deletes everything except pkgar files)
    pub clean_target: Option<bool>,
    /// skip recipe source updates during fetch
    pub static_source: Option<bool>,
}

#[derive(Debug, Default, Clone, Deserialize, PartialEq, Serialize)]
pub struct CookConfig {
    pub offline: bool,
    pub jobs: usize,
    pub tui: bool,
    pub logs: bool,
    pub nonstop: bool,
    pub verbose: bool,
    pub clean_build: bool,
    pub clean_target: bool,
    pub static_source: bool,
}

impl From<CookConfigOpt> for CookConfig {
    fn from(value: CookConfigOpt) -> Self {
        CookConfig {
            offline: value.offline.unwrap(),
            jobs: value.jobs.unwrap(),
            tui: value.tui.unwrap(),
            logs: value.logs.unwrap(),
            nonstop: value.nonstop.unwrap(),
            verbose: value.verbose.unwrap(),
            clean_build: value.clean_build.unwrap(),
            clean_target: value.clean_target.unwrap(),
            static_source: value.static_source.unwrap(),
        }
    }
}

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct CookbookConfig {
    #[serde(rename = "cook")]
    cook_opt: CookConfigOpt,
    #[serde(skip)]
    pub cook: CookConfig,
    pub mirrors: HashMap<String, String>,
}

static CONFIG: OnceLock<CookbookConfig> = OnceLock::new();

pub fn init_config() {
    let mut config: CookbookConfig = if fs::exists("cookbook.toml").unwrap_or(false) {
        let toml_content = fs::read_to_string("cookbook.toml")
            .map_err(|e| format!("Unable to read config: {:?}", e))
            .unwrap();
        toml::from_str(&toml_content)
            .map_err(|e| format!("Unable to parse config: {:?}", e))
            .unwrap()
    } else {
        CookbookConfig::default()
    };

    if config.cook_opt.tui.is_none() {
        config.cook_opt.tui = Some(!env::var("CI").is_ok_and(|s| !s.is_empty()));
    }
    if config.cook_opt.jobs.is_none() {
        config.cook_opt.jobs = Some(extract_env(
            "COOKBOOK_MAKE_JOBS",
            std::thread::available_parallelism()
                .map(|f| usize::from(f))
                .unwrap_or(1),
        ));
    }
    if config.cook_opt.logs.is_none() {
        config.cook_opt.logs = Some(extract_env("COOKBOOK_LOGS", config.cook_opt.tui.unwrap()));
    }
    if config.cook_opt.offline.is_none() {
        config.cook_opt.offline = Some(extract_env("COOKBOOK_OFFLINE", false));
    }
    if config.cook_opt.verbose.is_none() {
        config.cook_opt.verbose = Some(extract_env("COOKBOOK_VERBOSE", true));
    }
    if config.cook_opt.nonstop.is_none() {
        config.cook_opt.nonstop = Some(extract_env("COOKBOOK_NONSTOP", false));
    }
    if config.cook_opt.clean_build.is_none() {
        config.cook_opt.clean_build = Some(extract_env("COOKBOOK_CLEAN_BUILD", false));
    }
    if config.cook_opt.clean_target.is_none() {
        config.cook_opt.clean_target = Some(extract_env("COOKBOOK_CLEAN_TARGET", false));
    }
    if config.cook_opt.static_source.is_none() {
        config.cook_opt.static_source = Some(extract_env("COOKBOOK_STATIC_SOURCE", false));
    }
    if config.mirrors.len() == 0 {
        // The GNU FTP mirror below is automatically inserted for convenience
        // You can choose other mirrors by setting it on cookbook.toml
        config.mirrors.insert(
            "ftp.gnu.org/gnu".to_string(),
            "mirrors.ocf.berkeley.edu/gnu".to_string(),
        );
    }

    config.cook = CookConfig::from(config.cook_opt.clone());

    CONFIG.set(config).expect("config is initialized twice");
}

fn extract_env<T: FromStr>(key: &str, default: T) -> T {
    if let Ok(e) = env::var(&key) {
        str::parse(&e).unwrap_or(default)
    } else {
        default
    }
}

pub fn get_config() -> &'static CookbookConfig {
    return CONFIG.get().expect("Configuration is not initialized");
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
        let app_config = toml::from_str(
            "[mirrors]\n\
            \"ftp.gnu.org/gnu\" = \"example.com/gnu\"\n\
            \"github.com/foo/bar\" = \"github.com/baz/bar\"\n\
            \"github.com/a\" = \"github.com/b\"\n",
        )
        .expect("Unable to parse test config");
        // This will be called for each test. If the config is already set,
        // it will do nothing, which is fine as all tests use the same config.
        let _ = CONFIG.set(app_config);
    }

    #[test]
    fn test_parse_cook() {
        let app_config: CookbookConfig = toml::from_str(
            "[cook]\n\
            offline = true\n",
        )
        .expect("Unable to parse test config");
        assert_eq!(app_config.cook_opt.offline, Some(true));
        assert_eq!(app_config.cook_opt.jobs, None);
    }

    #[test]
    fn test_exact_match() {
        setup_test_config();
        assert_eq!(translate_mirror("ftp.gnu.org/gnu"), "example.com/gnu");
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
            translate_mirror("https://ftp.gnu.org/gnu/bash/bash-5.2.15.tar.gz"),
            "https://example.com/gnu/bash/bash-5.2.15.tar.gz"
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
