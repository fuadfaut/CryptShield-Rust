use crate::core::command_plan::CommandPlan;
use crate::core::config_manager::{
    update_config_contents, ConfigUpdateError, DnscryptConfig, DEFAULT_DNSCRYPT_CONFIG_PATH,
};
use crate::core::system_helper::{HelperAction, HelperRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemActionPlan {
    pub commands: Vec<CommandPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigUpdatePlan {
    pub path: String,
    pub contents: String,
}

pub fn build_system_action_plan(request: &HelperRequest) -> SystemActionPlan {
    let mut commands = Vec::new();

    match request.action {
        HelperAction::Start => {
            commands.push(command("systemctl", &["enable", "--now", "dnscrypt-proxy"]));
            add_local_dns_connection_commands(&mut commands, &request.connection_uuids);
        }
        HelperAction::Stop => {
            commands.push(command(
                "systemctl",
                &["disable", "--now", "dnscrypt-proxy"],
            ));
            add_restore_dns_connection_commands(&mut commands, &request.connection_uuids);
        }
        HelperAction::Restart => {
            commands.push(command("systemctl", &["restart", "dnscrypt-proxy"]));
            add_local_dns_connection_commands(&mut commands, &request.connection_uuids);
        }
    }

    SystemActionPlan { commands }
}

pub fn build_config_update_plan(
    existing_contents: &str,
    request: &HelperRequest,
) -> Result<Option<ConfigUpdatePlan>, ConfigUpdateError> {
    if request.action == HelperAction::Stop {
        return Ok(None);
    }

    let desired = DnscryptConfig {
        resolver_id: request.resolver_id.clone(),
        cache_enabled: request.cache_enabled,
        dnssec_required: request.dnssec_required,
    };
    let contents = update_config_contents(existing_contents, &desired)?;

    Ok(Some(ConfigUpdatePlan {
        path: DEFAULT_DNSCRYPT_CONFIG_PATH.to_owned(),
        contents,
    }))
}

fn add_local_dns_connection_commands(commands: &mut Vec<CommandPlan>, connection_uuids: &[String]) {
    for uuid in connection_uuids {
        commands.push(command(
            "nmcli",
            &[
                "--wait",
                "15",
                "connection",
                "modify",
                uuid,
                "ipv4.dns",
                "127.0.0.1",
                "ipv4.ignore-auto-dns",
                "yes",
            ],
        ));
        commands.push(command(
            "nmcli",
            &["--wait", "15", "connection", "up", uuid],
        ));
    }
}

fn add_restore_dns_connection_commands(
    commands: &mut Vec<CommandPlan>,
    connection_uuids: &[String],
) {
    for uuid in connection_uuids {
        commands.push(command(
            "nmcli",
            &[
                "--wait",
                "15",
                "connection",
                "modify",
                uuid,
                "ipv4.ignore-auto-dns",
                "no",
                "ipv4.dns",
                "",
            ],
        ));
        commands.push(command(
            "nmcli",
            &["--wait", "15", "connection", "up", uuid],
        ));
    }
}

fn command(program: &str, args: &[&str]) -> CommandPlan {
    CommandPlan {
        program: program.to_owned(),
        args: args.iter().map(|value| value.to_string()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::core::command_plan::CommandPlan;
    use crate::core::config_manager::parse_config;
    use crate::core::system_helper::parse_helper_request;

    use super::{build_config_update_plan, build_system_action_plan};

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn command(program: &str, args: &[&str]) -> CommandPlan {
        CommandPlan {
            program: program.to_owned(),
            args: args.iter().map(|value| value.to_string()).collect(),
        }
    }

    #[test]
    fn builds_start_plan_with_structured_commands() {
        let request = parse_helper_request(&args(&[
            "start",
            "cloudflare",
            "true",
            "true",
            "123e4567-e89b-12d3-a456-426614174000",
        ]))
        .expect("valid request");

        let plan = build_system_action_plan(&request);

        assert_eq!(
            plan.commands,
            vec![
                command("systemctl", &["enable", "--now", "dnscrypt-proxy"]),
                command(
                    "nmcli",
                    &[
                        "--wait",
                        "15",
                        "connection",
                        "modify",
                        "123e4567-e89b-12d3-a456-426614174000",
                        "ipv4.dns",
                        "127.0.0.1",
                        "ipv4.ignore-auto-dns",
                        "yes",
                    ],
                ),
                command(
                    "nmcli",
                    &[
                        "--wait",
                        "15",
                        "connection",
                        "up",
                        "123e4567-e89b-12d3-a456-426614174000",
                    ],
                ),
            ]
        );
    }

    #[test]
    fn builds_stop_plan_that_restores_networkmanager_dns() {
        let request =
            parse_helper_request(&args(&["stop", "123e4567-e89b-12d3-a456-426614174000"]))
                .expect("valid request");

        let plan = build_system_action_plan(&request);

        assert_eq!(
            plan.commands,
            vec![
                command("systemctl", &["disable", "--now", "dnscrypt-proxy"]),
                command(
                    "nmcli",
                    &[
                        "--wait",
                        "15",
                        "connection",
                        "modify",
                        "123e4567-e89b-12d3-a456-426614174000",
                        "ipv4.ignore-auto-dns",
                        "no",
                        "ipv4.dns",
                        "",
                    ],
                ),
                command(
                    "nmcli",
                    &[
                        "--wait",
                        "15",
                        "connection",
                        "up",
                        "123e4567-e89b-12d3-a456-426614174000",
                    ],
                ),
            ]
        );
    }

    #[test]
    fn builds_restart_plan_for_each_connection() {
        let request = parse_helper_request(&args(&[
            "restart",
            "quad9",
            "false",
            "true",
            "123e4567-e89b-12d3-a456-426614174000",
            "123e4567-e89b-12d3-a456-426614174111",
        ]))
        .expect("valid request");

        let plan = build_system_action_plan(&request);

        assert_eq!(plan.commands.len(), 5);
        assert_eq!(
            plan.commands[0],
            command("systemctl", &["restart", "dnscrypt-proxy"])
        );
        assert_eq!(
            plan.commands[1],
            command(
                "nmcli",
                &[
                    "--wait",
                    "15",
                    "connection",
                    "modify",
                    "123e4567-e89b-12d3-a456-426614174000",
                    "ipv4.dns",
                    "127.0.0.1",
                    "ipv4.ignore-auto-dns",
                    "yes",
                ],
            )
        );
        assert_eq!(
            plan.commands[3],
            command(
                "nmcli",
                &[
                    "--wait",
                    "15",
                    "connection",
                    "modify",
                    "123e4567-e89b-12d3-a456-426614174111",
                    "ipv4.dns",
                    "127.0.0.1",
                    "ipv4.ignore-auto-dns",
                    "yes",
                ],
            )
        );
    }

    #[test]
    fn builds_config_update_plan_for_restart_request() {
        let request = parse_helper_request(&args(&[
            "restart",
            "quad9",
            "false",
            "true",
            "123e4567-e89b-12d3-a456-426614174000",
        ]))
        .expect("valid request");

        let plan = build_config_update_plan(
            "server_names = ['cloudflare']\ncache = true\nrequire_dnssec = false\n",
            &request,
        )
        .expect("config update plan")
        .expect("restart updates config");

        assert_eq!(plan.path, "/etc/dnscrypt-proxy/dnscrypt-proxy.toml");

        let parsed = parse_config(&plan.contents).expect("valid updated config");
        let config = parsed.config.expect("config");
        assert_eq!(config.resolver_id.as_deref(), Some("quad9"));
        assert_eq!(config.cache_enabled, Some(false));
        assert_eq!(config.dnssec_required, Some(true));
    }

    #[test]
    fn does_not_build_config_update_for_stop_request() {
        let request =
            parse_helper_request(&args(&["stop", "123e4567-e89b-12d3-a456-426614174000"]))
                .expect("valid request");

        let plan = build_config_update_plan("server_names = ['cloudflare']\n", &request)
            .expect("config plan result");

        assert_eq!(plan, None);
    }
}
