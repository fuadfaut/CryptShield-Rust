#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolver {
    pub id: &'static str,
    pub name: &'static str,
}

pub const DEFAULT_RESOLVER_ID: &str = "cloudflare";

const RESOLVERS: &[Resolver] = &[
    Resolver {
        id: "cloudflare",
        name: "Cloudflare",
    },
    Resolver {
        id: "quad9",
        name: "Quad9",
    },
    Resolver {
        id: "adguard-dns",
        name: "AdGuard DNS",
    },
    Resolver {
        id: "google",
        name: "Google",
    },
];

pub fn find(id: &str) -> Option<&'static Resolver> {
    RESOLVERS.iter().find(|resolver| resolver.id == id)
}

pub fn is_supported(id: &str) -> bool {
    find(id).is_some()
}

pub fn display_name(id: &str) -> &'static str {
    find(id).map_or("Unknown", |resolver| resolver.name)
}

#[cfg(test)]
mod tests {
    use super::{display_name, is_supported, DEFAULT_RESOLVER_ID};

    #[test]
    fn validates_supported_resolvers() {
        assert!(is_supported(DEFAULT_RESOLVER_ID));
        assert!(is_supported("quad9"));
        assert!(!is_supported("bash -c reboot"));
    }

    #[test]
    fn maps_display_names() {
        assert_eq!(display_name("cloudflare"), "Cloudflare");
        assert_eq!(display_name("unknown"), "Unknown");
    }
}
