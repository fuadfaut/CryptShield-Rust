use crate::core::system_helper::{HelperAction, HelperRequest};

pub const CRYPTSHIELD_BINARY: &str = "/usr/bin/cryptshield";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPlan {
    pub program: String,
    pub args: Vec<String>,
}

pub fn pkexec_helper_plan(request: &HelperRequest) -> CommandPlan {
    let mut args = vec![
        CRYPTSHIELD_BINARY.to_owned(),
        "--system-helper".to_owned(),
        action_name(request.action).to_owned(),
    ];

    if let Some(resolver_id) = &request.resolver_id {
        args.push(resolver_id.clone());
    }

    if let Some(cache_enabled) = request.cache_enabled {
        args.push(cache_enabled.to_string());
    }

    if let Some(dnssec_required) = request.dnssec_required {
        args.push(dnssec_required.to_string());
    }

    args.extend(request.connection_uuids.iter().cloned());

    CommandPlan {
        program: "pkexec".to_owned(),
        args,
    }
}

fn action_name(action: HelperAction) -> &'static str {
    match action {
        HelperAction::Start => "start",
        HelperAction::Stop => "stop",
        HelperAction::Restart => "restart",
    }
}

#[cfg(test)]
mod tests {
    use crate::core::system_helper::{parse_helper_request, HelperAction};

    use super::{pkexec_helper_plan, CRYPTSHIELD_BINARY};

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn builds_structured_start_plan() {
        let request = parse_helper_request(&args(&[
            "start",
            "cloudflare",
            "true",
            "true",
            "123e4567-e89b-12d3-a456-426614174000",
        ]))
        .expect("validated request");

        let plan = pkexec_helper_plan(&request);

        assert_eq!(request.action, HelperAction::Start);
        assert_eq!(plan.program, "pkexec");
        assert_eq!(
            plan.args,
            args(&[
                CRYPTSHIELD_BINARY,
                "--system-helper",
                "start",
                "cloudflare",
                "true",
                "true",
                "123e4567-e89b-12d3-a456-426614174000",
            ])
        );
    }

    #[test]
    fn builds_structured_stop_plan() {
        let request =
            parse_helper_request(&args(&["stop", "123e4567-e89b-12d3-a456-426614174000"]))
                .expect("validated request");

        let plan = pkexec_helper_plan(&request);

        assert_eq!(
            plan.args,
            args(&[
                CRYPTSHIELD_BINARY,
                "--system-helper",
                "stop",
                "123e4567-e89b-12d3-a456-426614174000",
            ])
        );
    }
}
