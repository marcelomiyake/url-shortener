# URL Shortener database model

This is the human-readable model of the Cassandra mapping table. The CQL schema file is authoritative; update this inventory when it changes.

- **Owner:** `url-shortener`, `url-shortener-api` and [`database/cassandra/`](../database/cassandra/).
- **Known application consumer:** `url-shortener` / `url-shortener-api` reads and writes mappings. `url-shortener` / `url-shortener-frontend` consumes the API and does not access Cassandra directly. No cross-repository database consumer is identified.
- **Database:** Apache Cassandra-compatible CQL, keyspace substituted by the deployment configuration.
- **Source:** [`short_links_by_code.cql`](../database/cassandra/schema/short_links_by_code.cql).

## Tables

### `{keyspace}.short_links_by_code`

| Column | Type and rules | Description |
| --- | --- | --- |
| `code` | `text`, primary key and partition key | Public short code used to look up a destination. |
| `canonical_url` | `text`, nullable | Canonical HTTP(S) destination URL associated with the code. |
| `created_at` | `timestamp`, nullable | Mapping creation time. |

Cassandra uses the primary key as the lookup key; the schema declares no clustering columns, secondary indexes, or additional constraints. API code applies destination validation and uses conditional writes for code allocation; those behaviors are application-level guarantees, not CQL table constraints.

## Related documentation

- [URL Shortener System Design](design/url-shortener/design.md)
- [Contract catalog](contracts/README.md)
- [Documentation index](README.md)
