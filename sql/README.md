# SQL

We need a torrents table to store the last time an infohash was announce

```sql
CREATE TABLE torrents (
    infohash BYTEA NOT NULL PRIMARY KEY,
    last_announce BIGINT NOT NULL
);
```