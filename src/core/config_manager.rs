use std::fs;
use std::path::Path;

use crate::core::resolvers;
use toml::Value;
use toml_edit::{value, Array, DocumentMut};

pub const DEFAULT_DNSCRYPT_CONFIG_PATH: &str = "/etc/dnscrypt-proxy/dnscrypt-proxy.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnscryptConfig {
    pub resolver_id: Option<String>,
    pub cache_enabled: Option<bool>,
    pub dnssec_required: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigReadResult {
    pub status: String,
    pub config: Option<DnscryptConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigUpdateError {
    ParseError,
    UnsupportedResolver(String),
}

pub fn read_default_config() -> ConfigReadResult {
    read_config(Path::new(DEFAULT_DNSCRYPT_CONFIG_PATH))
}

pub fn read_config(path: &Path) -> ConfigReadResult {
    match fs::read_to_string(path) {
        Ok(contents) => parse_config(&contents).unwrap_or_else(|_| ConfigReadResult {
            status: "parse error".to_owned(),
            config: None,
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ConfigReadResult {
            status: "not found".to_owned(),
            config: None,
        },
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => ConfigReadResult {
            status: "permission denied".to_owned(),
            config: None,
        },
        Err(_) => ConfigReadResult {
            status: "unreadable".to_owned(),
            config: None,
        },
    }
}

pub fn parse_config(contents: &str) -> Result<ConfigReadResult, toml::de::Error> {
    let value = toml::from_str::<Value>(contents)?;
    let resolver_id = value
        .get("server_names")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(Value::as_str)
        .map(str::to_owned);

    let cache_enabled = value.get("cache").and_then(Value::as_bool);
    let dnssec_required = value.get("require_dnssec").and_then(Value::as_bool);

    Ok(ConfigReadResult {
        status: "loaded".to_owned(),
        config: Some(DnscryptConfig {
            resolver_id,
            cache_enabled,
            dnssec_required,
        }),
    })
}

pub fn update_config_contents(
    contents: &str,
    desired: &DnscryptConfig,
) -> Result<String, ConfigUpdateError> {
    let mut document = contents
        .parse::<DocumentMut>()
        .map_err(|_| ConfigUpdateError::ParseError)?;

    if let Some(resolver_id) = &desired.resolver_id {
        if !resolvers::is_supported(resolver_id) {
            return Err(ConfigUpdateError::UnsupportedResolver(resolver_id.clone()));
        }

        let mut server_names = Array::default();
        server_names.push(resolver_id.as_str());
        document["server_names"] = value(server_names);
    }

    if let Some(cache_enabled) = desired.cache_enabled {
        document["cache"] = value(cache_enabled);
    }

    if let Some(dnssec_required) = desired.dnssec_required {
        document["require_dnssec"] = value(dnssec_required);
    }

    Ok(document.to_string())
}

#[cfg(test)]
mod tests {
    use super::{parse_config, update_config_contents, ConfigUpdateError, DnscryptConfig};

    #[test]
    fn reads_top_level_dnscrypt_options() {
        let result = parse_config(
            r#"
server_names = ['quad9']
cache = true
require_dnssec = true
"#,
        )
        .expect("valid toml");

        let config = result.config.expect("config");
        assert_eq!(result.status, "loaded");
        assert_eq!(config.resolver_id.as_deref(), Some("quad9"));
        assert_eq!(config.cache_enabled, Some(true));
        assert_eq!(config.dnssec_required, Some(true));
    }

    #[test]
    fn updates_dnscrypt_options_with_toml_editor() {
        let updated = update_config_contents(
            r#"
# keep this comment
server_names = ['cloudflare']
cache = false
require_dnssec = false
"#,
            &DnscryptConfig {
                resolver_id: Some("quad9".to_owned()),
                cache_enabled: Some(true),
                dnssec_required: Some(true),
            },
        )
        .expect("updated config");

        assert!(updated.contains("# keep this comment"));

        let parsed = parse_config(&updated).expect("valid updated toml");
        let config = parsed.config.expect("config");
        assert_eq!(config.resolver_id.as_deref(), Some("quad9"));
        assert_eq!(config.cache_enabled, Some(true));
        assert_eq!(config.dnssec_required, Some(true));
    }

    #[test]
    fn rejects_unsupported_resolver_when_updating_config() {
        let error = update_config_contents(
            "server_names = ['cloudflare']\n",
            &DnscryptConfig {
                resolver_id: Some("cloudflare; reboot".to_owned()),
                cache_enabled: None,
                dnssec_required: None,
            },
        )
        .expect_err("unsupported resolver");

        assert_eq!(
            error,
            ConfigUpdateError::UnsupportedResolver("cloudflare; reboot".to_owned())
        );
    }
}
