use crate::core::resolvers;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelperExecutionError {
    RequiresRoot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelperAction {
    Start,
    Stop,
    Restart,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelperRequest {
    pub action: HelperAction,
    pub resolver_id: Option<String>,
    pub cache_enabled: Option<bool>,
    pub dnssec_required: Option<bool>,
    pub connection_uuids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelperParseError {
    MissingAction,
    UnknownAction(String),
    MissingResolver,
    UnsupportedResolver(String),
    MissingBoolean(&'static str),
    InvalidBoolean(String),
    MissingConnection,
    InvalidConnectionUuid(String),
    UnexpectedArgumentCount,
}

pub fn current_effective_uid() -> Option<u32> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    parse_effective_uid_from_status(&status)
}

pub fn helper_execution_allowed(effective_uid: u32) -> Result<(), HelperExecutionError> {
    if effective_uid == 0 {
        Ok(())
    } else {
        Err(HelperExecutionError::RequiresRoot)
    }
}

pub fn parse_helper_request(args: &[String]) -> Result<HelperRequest, HelperParseError> {
    let Some(action) = args.first() else {
        return Err(HelperParseError::MissingAction);
    };

    match action.as_str() {
        "start" => parse_start_or_restart(HelperAction::Start, &args[1..]),
        "restart" => parse_start_or_restart(HelperAction::Restart, &args[1..]),
        "stop" => parse_stop(&args[1..]),
        value => Err(HelperParseError::UnknownAction(value.to_owned())),
    }
}

fn parse_effective_uid_from_status(status: &str) -> Option<u32> {
    let uid_line = status.lines().find(|line| line.starts_with("Uid:"))?;
    let effective_uid = uid_line.split_whitespace().nth(2)?;
    effective_uid.parse().ok()
}

fn parse_start_or_restart(
    action: HelperAction,
    args: &[String],
) -> Result<HelperRequest, HelperParseError> {
    if args.len() < 4 {
        return Err(HelperParseError::UnexpectedArgumentCount);
    }

    let resolver_id = args
        .first()
        .ok_or(HelperParseError::MissingResolver)?
        .to_owned();
    if !resolvers::is_supported(&resolver_id) {
        return Err(HelperParseError::UnsupportedResolver(resolver_id));
    }

    let cache_enabled = parse_bool(args.get(1), "cache_enabled")?;
    let dnssec_required = parse_bool(args.get(2), "dnssec_required")?;
    let connection_uuids = parse_connection_uuids(&args[3..])?;

    Ok(HelperRequest {
        action,
        resolver_id: Some(resolver_id),
        cache_enabled: Some(cache_enabled),
        dnssec_required: Some(dnssec_required),
        connection_uuids,
    })
}

fn parse_stop(args: &[String]) -> Result<HelperRequest, HelperParseError> {
    let connection_uuids = parse_connection_uuids(args)?;

    Ok(HelperRequest {
        action: HelperAction::Stop,
        resolver_id: None,
        cache_enabled: None,
        dnssec_required: None,
        connection_uuids,
    })
}

fn parse_bool(value: Option<&String>, name: &'static str) -> Result<bool, HelperParseError> {
    match value.map(String::as_str) {
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        Some(value) => Err(HelperParseError::InvalidBoolean(value.to_owned())),
        None => Err(HelperParseError::MissingBoolean(name)),
    }
}

fn parse_connection_uuids(args: &[String]) -> Result<Vec<String>, HelperParseError> {
    if args.is_empty() {
        return Err(HelperParseError::MissingConnection);
    }

    let mut uuids = Vec::with_capacity(args.len());
    for arg in args {
        if !is_valid_uuid_arg(arg) {
            return Err(HelperParseError::InvalidConnectionUuid(arg.to_owned()));
        }
        uuids.push(arg.to_owned());
    }

    Ok(uuids)
}

fn is_valid_uuid_arg(value: &str) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    let expected = [8, 4, 4, 4, 12];

    parts.len() == expected.len()
        && parts
            .iter()
            .zip(expected)
            .all(|(part, len)| part.len() == len && part.chars().all(|ch| ch.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::{
        helper_execution_allowed, parse_effective_uid_from_status, parse_helper_request,
        HelperAction, HelperExecutionError, HelperParseError,
    };

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parses_start_request() {
        let request = parse_helper_request(&args(&[
            "start",
            "cloudflare",
            "true",
            "false",
            "123e4567-e89b-12d3-a456-426614174000",
        ]))
        .expect("valid helper request");

        assert_eq!(request.action, HelperAction::Start);
        assert_eq!(request.resolver_id.as_deref(), Some("cloudflare"));
        assert_eq!(request.cache_enabled, Some(true));
        assert_eq!(request.dnssec_required, Some(false));
        assert_eq!(request.connection_uuids.len(), 1);
    }

    #[test]
    fn parses_stop_request() {
        let request =
            parse_helper_request(&args(&["stop", "123e4567-e89b-12d3-a456-426614174000"]))
                .expect("valid helper request");

        assert_eq!(request.action, HelperAction::Stop);
        assert_eq!(request.resolver_id, None);
    }

    #[test]
    fn rejects_shell_like_resolver() {
        let error = parse_helper_request(&args(&[
            "start",
            "cloudflare;reboot",
            "true",
            "true",
            "123e4567-e89b-12d3-a456-426614174000",
        ]))
        .expect_err("unsupported resolver");

        assert_eq!(
            error,
            HelperParseError::UnsupportedResolver("cloudflare;reboot".to_owned())
        );
    }

    #[test]
    fn rejects_non_exact_booleans() {
        let error = parse_helper_request(&args(&[
            "restart",
            "quad9",
            "yes",
            "true",
            "123e4567-e89b-12d3-a456-426614174000",
        ]))
        .expect_err("invalid bool");

        assert_eq!(error, HelperParseError::InvalidBoolean("yes".to_owned()));
    }

    #[test]
    fn rejects_invalid_connection_uuid() {
        let error = parse_helper_request(&args(&["stop", "$(nmcli connection show)"]))
            .expect_err("invalid uuid");

        assert_eq!(
            error,
            HelperParseError::InvalidConnectionUuid("$(nmcli connection show)".to_owned())
        );
    }

    #[test]
    fn allows_helper_execution_only_for_root() {
        assert_eq!(helper_execution_allowed(0), Ok(()));
        assert_eq!(
            helper_execution_allowed(1000),
            Err(HelperExecutionError::RequiresRoot)
        );
    }

    #[test]
    fn parses_effective_uid_from_proc_status() {
        let uid = parse_effective_uid_from_status(
            "\
Name:\tcryptshield
Uid:\t1000\t1001\t1002\t1003
",
        );

        assert_eq!(uid, Some(1001));
    }
}
