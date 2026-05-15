#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolver {
    pub id: &'static str,
    pub config_name: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

pub const DEFAULT_RESOLVER_ID: &str = "default";

const RESOLVERS: &[Resolver] = &[
    Resolver {
        id: "default",
        config_name: "",
        name: "Default",
        description: "dnscrypt-proxy load-balanced default resolver set.",
    },
    Resolver {
        id: "cloudflare",
        config_name: "cloudflare",
        name: "Cloudflare",
        description: "Fast general-purpose DNSCrypt resolver.",
    },
    Resolver {
        id: "google",
        config_name: "google",
        name: "Google",
        description: "High-availability public DNS resolver.",
    },
    Resolver {
        id: "quad9",
        config_name: "quad9",
        name: "Quad9",
        description: "Security-focused resolver with threat blocking.",
    },
    Resolver {
        id: "adguard",
        config_name: "adguard",
        name: "AdGuard DNS",
        description: "Privacy resolver with ad and tracker filtering.",
    },
    Resolver {
        id: "nextdns",
        config_name: "nextdns",
        name: "NextDNS",
        description: "Configurable privacy-focused resolver.",
    },
    Resolver {
        id: "cisco",
        config_name: "cisco",
        name: "Cisco",
        description: "Security-oriented public resolver.",
    },
    Resolver {
        id: "mullvad",
        config_name: "mullvad-doh",
        name: "Mullvad",
        description: "Privacy-first DNS-over-HTTPS resolver.",
    },
    Resolver {
        id: "cleanbrowsing",
        config_name: "cleanbrowsing-adult",
        name: "CleanBrowsing",
        description: "Family/adult-content filtering resolver.",
    },
    Resolver {
        id: "tiarapp",
        config_name: "doh.tiar.app",
        name: "Tiarap DNS",
        description: "Community DNS-over-HTTPS resolver.",
    },
];

#[cfg(test)]
pub fn all() -> &'static [Resolver] {
    RESOLVERS
}

pub fn find(id: &str) -> Option<&'static Resolver> {
    RESOLVERS.iter().find(|resolver| resolver.id == id)
}

pub fn find_by_config_name(config_name: &str) -> Option<&'static Resolver> {
    RESOLVERS
        .iter()
        .find(|resolver| resolver.config_name == config_name)
}

pub fn find_supported(value: &str) -> Option<&'static Resolver> {
    find(value).or_else(|| find_by_config_name(value))
}

pub fn is_supported(id: &str) -> bool {
    find_supported(id).is_some()
}

pub fn display_name(id: &str) -> &'static str {
    find_supported(id).map_or("Unknown", |resolver| resolver.name)
}

pub fn config_name(id: &str) -> Option<&'static str> {
    find_supported(id).map(|resolver| resolver.config_name)
}

pub fn ui_id_for_config_name(config_name: Option<&str>) -> &'static str {
    config_name
        .and_then(find_by_config_name)
        .map_or(DEFAULT_RESOLVER_ID, |resolver| resolver.id)
}

#[cfg(test)]
mod tests {
    use super::{
        all, config_name, display_name, is_supported, ui_id_for_config_name, DEFAULT_RESOLVER_ID,
    };

    #[test]
    fn matches_original_cryptshield_readme_resolvers() {
        let config_names = all()
            .iter()
            .map(|resolver| resolver.config_name)
            .collect::<Vec<_>>();

        assert_eq!(
            config_names,
            vec![
                "",
                "cloudflare",
                "google",
                "quad9",
                "adguard",
                "nextdns",
                "cisco",
                "mullvad-doh",
                "cleanbrowsing-adult",
                "doh.tiar.app",
            ]
        );
    }

    #[test]
    fn validates_supported_resolver_ids_and_config_names() {
        assert!(is_supported(DEFAULT_RESOLVER_ID));
        assert!(is_supported(""));
        assert!(is_supported("quad9"));
        assert!(is_supported("mullvad"));
        assert!(is_supported("mullvad-doh"));
        assert!(!is_supported("bash -c reboot"));
    }

    #[test]
    fn maps_display_names() {
        assert_eq!(display_name("cloudflare"), "Cloudflare");
        assert_eq!(display_name("mullvad-doh"), "Mullvad");
        assert_eq!(display_name("unknown"), "Unknown");
    }

    #[test]
    fn maps_between_ui_ids_and_dnscrypt_config_names() {
        assert_eq!(config_name("default"), Some(""));
        assert_eq!(config_name("mullvad"), Some("mullvad-doh"));
        assert_eq!(ui_id_for_config_name(None), "default");
        assert_eq!(ui_id_for_config_name(Some("mullvad-doh")), "mullvad");
    }
}
