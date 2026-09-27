CREATE TABLE IF NOT EXISTS {keyspace}.short_links_by_code (
    code text PRIMARY KEY,
    canonical_url text,
    created_at timestamp
);
