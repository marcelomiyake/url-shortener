use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use scylla::{
    client::{
        execution_profile::ExecutionProfile, session::Session, session_builder::SessionBuilder,
    },
    policies::load_balancing::DefaultPolicy,
    statement::{Consistency, SerialConsistency, prepared::PreparedStatement},
};

use crate::{
    config::Settings,
    short_link::{
        domain::{DestinationUrl, ShortCode},
        service::{InsertOutcome, ShortLinkRepository, StorageError},
    },
};

const CREATE_KEYSPACE: &str = "CREATE KEYSPACE IF NOT EXISTS {keyspace} WITH replication = \
     {'class': 'NetworkTopologyStrategy', '{datacenter}': {replication_factor}}";
const SHORT_LINK_TABLE: &str =
    include_str!("../../../database/cassandra/schema/short_links_by_code.cql");

#[derive(Clone)]
pub struct CassandraRepository {
    session: Arc<Session>,
    insert_if_absent: Arc<PreparedStatement>,
    select_by_code: Arc<PreparedStatement>,
    readiness_check: Arc<PreparedStatement>,
}

impl CassandraRepository {
    pub async fn connect(settings: &Settings) -> Result<Self, StorageError> {
        let policy = DefaultPolicy::builder()
            .prefer_datacenter(settings.cassandra_local_dc.clone())
            .build();
        let profile = ExecutionProfile::builder()
            .consistency(Consistency::LocalQuorum)
            .load_balancing_policy(policy)
            .request_timeout(Some(Duration::from_secs(10)))
            .build();
        let session = SessionBuilder::new()
            .known_nodes(settings.cassandra_contact_points.iter())
            .default_execution_profile_handle(profile.into_handle())
            .build()
            .await
            .map_err(|_| StorageError)?;

        create_schema(&session, settings).await?;

        let keyspace = &settings.cassandra_keyspace;
        let mut insert_if_absent = session
            .prepare(format!(
                "INSERT INTO {keyspace}.short_links_by_code \
                 (code, canonical_url, created_at) VALUES (?, ?, ?) IF NOT EXISTS"
            ))
            .await
            .map_err(|_| StorageError)?;
        insert_if_absent.set_consistency(Consistency::LocalQuorum);
        insert_if_absent.set_serial_consistency(Some(SerialConsistency::LocalSerial));

        let mut select_by_code = session
            .prepare(format!(
                "SELECT canonical_url FROM {keyspace}.short_links_by_code WHERE code = ?"
            ))
            .await
            .map_err(|_| StorageError)?;
        select_by_code.set_consistency(Consistency::LocalQuorum);

        let mut readiness_check = session
            .prepare(format!(
                "SELECT code FROM {keyspace}.short_links_by_code WHERE code = ?"
            ))
            .await
            .map_err(|_| StorageError)?;
        readiness_check.set_consistency(Consistency::LocalQuorum);
        readiness_check.set_request_timeout(Some(Duration::from_secs(1)));

        Ok(Self {
            session: Arc::new(session),
            insert_if_absent: Arc::new(insert_if_absent),
            select_by_code: Arc::new(select_by_code),
            readiness_check: Arc::new(readiness_check),
        })
    }
}

impl ShortLinkRepository for CassandraRepository {
    async fn insert_if_absent(
        &self,
        code: &ShortCode,
        destination: &DestinationUrl,
        created_at: DateTime<Utc>,
    ) -> Result<InsertOutcome, StorageError> {
        let result = self
            .session
            .execute_unpaged(
                self.insert_if_absent.as_ref(),
                (code.as_str(), destination.as_str(), created_at),
            )
            .await
            .map_err(|_| StorageError)?
            .into_rows_result()
            .map_err(|_| StorageError)?;

        if result.column_specs().len() == 1 {
            let (applied,) = result.first_row::<(bool,)>().map_err(|_| StorageError)?;
            return applied
                .then_some(InsertOutcome::Inserted)
                .ok_or(StorageError);
        }

        let (applied, stored_code, stored_destination, _created_at) = result
            .first_row::<(bool, String, String, Option<DateTime<Utc>>)>()
            .map_err(|_| StorageError)?;
        if applied || stored_code != code.as_str() {
            return Err(StorageError);
        }

        Ok(InsertOutcome::AlreadyExists(stored_destination))
    }

    async fn find_by_code(&self, code: &ShortCode) -> Result<Option<String>, StorageError> {
        let result = self
            .session
            .execute_unpaged(self.select_by_code.as_ref(), (code.as_str(),))
            .await
            .map_err(|_| StorageError)?
            .into_rows_result()
            .map_err(|_| StorageError)?;

        result
            .maybe_first_row::<(String,)>()
            .map(|row| row.map(|(destination,)| destination))
            .map_err(|_| StorageError)
    }

    async fn is_ready(&self) -> Result<(), StorageError> {
        self.session
            .execute_unpaged(self.readiness_check.as_ref(), ("000000000000",))
            .await
            .map(|_| ())
            .map_err(|_| StorageError)
    }
}

async fn create_schema(session: &Session, settings: &Settings) -> Result<(), StorageError> {
    let create_keyspace = CREATE_KEYSPACE
        .replace("{keyspace}", &settings.cassandra_keyspace)
        .replace(
            "{datacenter}",
            &settings.cassandra_local_dc.replace('\'', "''"),
        )
        .replace(
            "{replication_factor}",
            &settings.cassandra_replication_factor.to_string(),
        );
    session
        .query_unpaged(create_keyspace, ())
        .await
        .map_err(|_| StorageError)?;

    let create_table = SHORT_LINK_TABLE.replace("{keyspace}", &settings.cassandra_keyspace);
    session
        .query_unpaged(create_table, ())
        .await
        .map_err(|_| StorageError)?;
    Ok(())
}
