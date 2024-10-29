# SQL

We need a torrents table to store the last time an infohash was announce

```sql
CREATE TABLE torrents (
    infohash BYTEA NOT NULL PRIMARY KEY,
    last_announce BIGINT NOT NULL,
    count BIGINT NOT NULL DEFAULT 1
);

CREATE INDEX last_announce_idx ON torrents(last_announce);
CREATE INDEX announce_count_idx ON torrents(count);

ALTER TABLE torrents ADD cleaned BOOLEAN NOT NULL DEFAULT FALSE;
```

## Query to get most announced torrents

```
SELECT encode(infohash, 'escape') as infohash, last_announce, count, cleaned FROM torrents ORDER BY count DESC LIMIT 10;
SELECT COUNT(*) FROM torrents;
SELECT COUNT(*) FROM torrents WHERE  last_announce > (extract(epoch from now()) * 1000) - 1000 * 60 * 31;
SELECT SUM(count) FROM torrents;
```

## Queries to get stale torrents

```
SELECT COUNT(*) FROM torrents WHERE last_announce < (extract(epoch from now()) * 1000) - 1000 * 60 * 31;
SELECT COUNT(*) FROM torrents WHERE  last_announce < (extract(epoch from now()) * 1000) - 1000 * 60 * 31 AND cleaned = TRUE;
SELECT COUNT(*) FROM torrents WHERE  last_announce < (extract(epoch from now()) * 1000) - 1000 * 60 * 31 AND cleaned = FALSE;
```

```
SELECT encode(infohash, 'escape') as infohash, last_announce, count, cleaned FROM torrents WHERE last_announce < (extract(epoch from now()) * 1000) - 1000 * 60 * 31  ORDER BY count DESC LIMIT 10;
```