use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveConnectionParseError {
    InvalidUuid(String),
}

pub fn active_connection_uuids() -> Result<Vec<String>, ActiveConnectionParseError> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "UUID,DEVICE", "connection", "show", "--active"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            parse_active_connection_uuids(&String::from_utf8_lossy(&output.stdout))
        }
        _ => Ok(Vec::new()),
    }
}

pub fn parse_active_connection_uuids(
    output: &str,
) -> Result<Vec<String>, ActiveConnectionParseError> {
    let mut uuids = Vec::new();

    for line in output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let mut fields = line.split(':');
        let uuid = fields.next().unwrap_or_default();
        let device = fields.next().unwrap_or_default();

        if device == "lo" {
            continue;
        }

        if !is_valid_uuid(uuid) {
            return Err(ActiveConnectionParseError::InvalidUuid(uuid.to_owned()));
        }
        uuids.push(uuid.to_owned());
    }

    Ok(uuids)
}

fn is_valid_uuid(value: &str) -> bool {
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
    use super::{parse_active_connection_uuids, ActiveConnectionParseError};

    #[test]
    fn parses_active_networkmanager_connection_uuids() {
        let uuids = parse_active_connection_uuids(
            "\
123e4567-e89b-12d3-a456-426614174000:wlp2s0
123e4567-e89b-12d3-a456-426614174111:enp3s0
123e4567-e89b-12d3-a456-426614174222:tun0
",
        )
        .expect("active connections");

        assert_eq!(
            uuids,
            vec![
                "123e4567-e89b-12d3-a456-426614174000",
                "123e4567-e89b-12d3-a456-426614174111",
                "123e4567-e89b-12d3-a456-426614174222",
            ]
        );
    }

    #[test]
    fn ignores_loopback_connection_from_original_helper_flow() {
        let uuids = parse_active_connection_uuids(
            "\
123e4567-e89b-12d3-a456-426614174000:wlp2s0
123e4567-e89b-12d3-a456-426614174111:lo
",
        )
        .expect("active connections");

        assert_eq!(uuids, vec!["123e4567-e89b-12d3-a456-426614174000"]);
    }

    #[test]
    fn ignores_empty_lines_and_extra_fields() {
        let uuids = parse_active_connection_uuids(
            "\n123e4567-e89b-12d3-a456-426614174000:wlp2s0:activated\n",
        )
        .expect("active connections");

        assert_eq!(uuids, vec!["123e4567-e89b-12d3-a456-426614174000"]);
    }

    #[test]
    fn rejects_invalid_uuid_output() {
        let error = parse_active_connection_uuids("$(bad):wlp2s0\n").expect_err("invalid uuid");

        assert_eq!(
            error,
            ActiveConnectionParseError::InvalidUuid("$(bad)".to_owned())
        );
    }
}
