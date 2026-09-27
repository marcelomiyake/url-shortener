use std::{env, net::SocketAddr, num::ParseIntError, str::FromStr};

pub struct Settings {
    pub listen_address: SocketAddr,
    pub cassandra_contact_points: Vec<String>,
    pub cassandra_keyspace: String,
    pub cassandra_local_dc: String,
    pub cassandra_replication_factor: u8,
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("required environment variable {0} is missing")]
    MissingVariable(&'static str),
    #[error("environment variable {0} has an invalid value")]
    InvalidVariable(&'static str),
}

impl Settings {
    pub fn from_env() -> Result<Self, SettingsError> {
        Self::from_lookup(|name| env::var(name).ok())
    }

    fn from_lookup(read: impl Fn(&str) -> Option<String>) -> Result<Self, SettingsError> {
        let listen_address = read("APP_LISTEN_ADDR")
            .unwrap_or_else(|| "0.0.0.0:8080".to_owned())
            .parse()
            .map_err(|_| SettingsError::InvalidVariable("APP_LISTEN_ADDR"))?;
        let cassandra_contact_points = parse_contact_points(
            "CASSANDRA_CONTACT_POINTS",
            required("CASSANDRA_CONTACT_POINTS", &read)?,
        )?;
        let cassandra_keyspace =
            read("CASSANDRA_KEYSPACE").unwrap_or_else(|| "url_shortener".to_owned());
        if !valid_keyspace(&cassandra_keyspace) {
            return Err(SettingsError::InvalidVariable("CASSANDRA_KEYSPACE"));
        }
        let cassandra_local_dc =
            read("CASSANDRA_LOCAL_DC").unwrap_or_else(|| "datacenter1".to_owned());
        if !valid_identifier(&cassandra_local_dc) {
            return Err(SettingsError::InvalidVariable("CASSANDRA_LOCAL_DC"));
        }
        let cassandra_replication_factor =
            parse_replication_factor("CASSANDRA_REPLICATION_FACTOR", "3", &read)?;

        Ok(Self {
            listen_address,
            cassandra_contact_points,
            cassandra_keyspace,
            cassandra_local_dc,
            cassandra_replication_factor,
        })
    }
}

fn required(
    name: &'static str,
    read: &impl Fn(&str) -> Option<String>,
) -> Result<String, SettingsError> {
    read(name).ok_or(SettingsError::MissingVariable(name))
}

fn parse_contact_points(name: &'static str, value: String) -> Result<Vec<String>, SettingsError> {
    let points = value
        .split(',')
        .map(str::trim)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if points.is_empty()
        || points.iter().any(|point| {
            let Some((host, port)) = point.rsplit_once(':') else {
                return true;
            };
            !valid_contact_host(host) || !port.parse::<u16>().is_ok_and(|parsed| parsed > 0)
        })
    {
        return Err(SettingsError::InvalidVariable(name));
    }

    Ok(points)
}

fn parse_replication_factor(
    name: &'static str,
    default: &str,
    read: &impl Fn(&str) -> Option<String>,
) -> Result<u8, SettingsError> {
    let value = read(name).unwrap_or_else(|| default.to_owned());
    let factor =
        u8::from_str(&value).map_err(|_: ParseIntError| SettingsError::InvalidVariable(name))?;
    if factor == 0 {
        return Err(SettingsError::InvalidVariable(name));
    }
    Ok(factor)
}

fn valid_keyspace(value: &str) -> bool {
    valid_identifier(value)
}

fn valid_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    matches!(bytes.next(), Some(b'a'..=b'z' | b'_'))
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_contact_host(host: &str) -> bool {
    !host.is_empty()
        && host
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Settings, SettingsError};

    fn settings(values: &HashMap<&str, &str>) -> Result<Settings, SettingsError> {
        Settings::from_lookup(|name| values.get(name).map(|value| (*value).to_owned()))
    }

    fn valid_values() -> HashMap<&'static str, &'static str> {
        HashMap::from([("CASSANDRA_CONTACT_POINTS", "localhost:9042")])
    }

    #[test]
    fn uses_defaults_for_optional_settings() {
        let settings = settings(&valid_values()).unwrap();

        assert_eq!(settings.listen_address.to_string(), "0.0.0.0:8080");
        assert_eq!(settings.cassandra_contact_points, ["localhost:9042"]);
        assert_eq!(settings.cassandra_keyspace, "url_shortener");
        assert_eq!(settings.cassandra_local_dc, "datacenter1");
        assert_eq!(settings.cassandra_replication_factor, 3);
    }

    #[test]
    fn accepts_explicit_optional_settings() {
        let values = HashMap::from([
            ("APP_LISTEN_ADDR", "127.0.0.1:9090"),
            (
                "CASSANDRA_CONTACT_POINTS",
                "cassandra-0:9042,cassandra-1:9042",
            ),
            ("CASSANDRA_KEYSPACE", "short_links"),
            ("CASSANDRA_LOCAL_DC", "datacenter1"),
            ("CASSANDRA_REPLICATION_FACTOR", "2"),
        ]);
        let settings = settings(&values).unwrap();

        assert_eq!(settings.listen_address.to_string(), "127.0.0.1:9090");
        assert_eq!(settings.cassandra_contact_points.len(), 2);
        assert_eq!(settings.cassandra_keyspace, "short_links");
        assert_eq!(settings.cassandra_replication_factor, 2);
    }

    #[test]
    fn rejects_invalid_listen_addresses_and_ports() {
        let mut values = valid_values();
        values.insert("APP_LISTEN_ADDR", "not-an-address");
        assert!(matches!(
            settings(&values),
            Err(SettingsError::InvalidVariable("APP_LISTEN_ADDR"))
        ));

        values.remove("APP_LISTEN_ADDR");
        values.insert("CASSANDRA_CONTACT_POINTS", "cassandra:not-a-port");
        assert!(matches!(
            settings(&values),
            Err(SettingsError::InvalidVariable("CASSANDRA_CONTACT_POINTS"))
        ));

        values.insert("CASSANDRA_CONTACT_POINTS", "localhost:9042");
        values.insert("CASSANDRA_CONTACT_POINTS", "bad host:9042");
        assert!(matches!(
            settings(&values),
            Err(SettingsError::InvalidVariable("CASSANDRA_CONTACT_POINTS"))
        ));

        values.insert("CASSANDRA_CONTACT_POINTS", "localhost:9042");
        values.insert("CASSANDRA_KEYSPACE", "Not-A-Keyspace");
        assert!(matches!(
            settings(&values),
            Err(SettingsError::InvalidVariable("CASSANDRA_KEYSPACE"))
        ));

        values.remove("CASSANDRA_KEYSPACE");
        values.insert("CASSANDRA_LOCAL_DC", "bad dc");
        assert!(matches!(
            settings(&values),
            Err(SettingsError::InvalidVariable("CASSANDRA_LOCAL_DC"))
        ));

        values.remove("CASSANDRA_LOCAL_DC");
        values.insert("CASSANDRA_REPLICATION_FACTOR", "0");
        assert!(matches!(
            settings(&values),
            Err(SettingsError::InvalidVariable(
                "CASSANDRA_REPLICATION_FACTOR"
            ))
        ));
    }

    #[test]
    fn requires_database_settings() {
        let name = "CASSANDRA_CONTACT_POINTS";
        let mut values = valid_values();
        values.remove(name);
        assert!(matches!(
            settings(&values),
            Err(SettingsError::MissingVariable(missing)) if missing == name
        ));
    }
}
