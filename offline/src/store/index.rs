use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use super::error::StoreError;
use super::types::Held;

const SCHEMA: &str = "
create table if not exists objects (
    hash text primary key,
    size integer not null,
    used_at integer not null
);
create table if not exists urls (
    url text primary key,
    hash text not null,
    kind text not null,
    fetched_at integer not null,
    etag text,
    last_modified text
);
create table if not exists holders (
    program text not null,
    url text not null,
    primary key (program, url)
);
create table if not exists programs (
    program text primary key,
    protected integer not null
);
";

#[derive(Debug)]
pub(super) struct Index {
    db: Connection,
}

impl Index {
    pub(super) fn open(path: &Path) -> Result<Self, StoreError> {
        let db = Connection::open(path)?;
        db.pragma_update(None, "journal_mode", "WAL")?;
        db.pragma_update(None, "synchronous", "FULL")?;
        db.execute_batch(SCHEMA)?;
        Ok(Self { db })
    }

    pub(super) fn remember(
        &mut self,
        url: &str,
        program: &str,
        held: &Held,
        at: i64,
    ) -> Result<(), StoreError> {
        let write = self.db.transaction()?;
        write.execute(
            "insert into objects (hash, size, used_at) values (?1, ?2, ?3)
             on conflict (hash) do update set used_at = ?3",
            params![held.hash, held.size, at],
        )?;
        write.execute(
            "insert into urls (url, hash, kind, fetched_at, etag, last_modified)
             values (?1, ?2, ?3, ?4, ?5, ?6)
             on conflict (url) do update set
                 hash = ?2, kind = ?3, fetched_at = ?4, etag = ?5, last_modified = ?6",
            params![url, held.hash, held.kind, at, held.etag, held.last_modified],
        )?;
        write.execute(
            "insert or ignore into holders (program, url) values (?1, ?2)",
            params![program, url],
        )?;
        write.commit()?;
        Ok(())
    }

    pub(super) fn find(&self, url: &str) -> Result<Option<Held>, StoreError> {
        let held = self
            .db
            .query_row(
                "select o.hash, o.size, u.kind, u.fetched_at, u.etag, u.last_modified
                 from urls u join objects o on o.hash = u.hash where u.url = ?1",
                params![url],
                |row| {
                    Ok(Held {
                        hash: row.get(0)?,
                        path: Path::new("").to_path_buf(),
                        size: row.get(1)?,
                        kind: row.get(2)?,
                        fetched_at: row.get(3)?,
                        etag: row.get(4)?,
                        last_modified: row.get(5)?,
                    })
                },
            )
            .optional()?;
        Ok(held)
    }

    pub(super) fn touch(&self, hash: &str, at: i64) -> Result<(), StoreError> {
        self.db.execute(
            "update objects set used_at = ?2 where hash = ?1",
            params![hash, at],
        )?;
        Ok(())
    }

    pub(super) fn protect(&self, program: &str, protected: bool) -> Result<(), StoreError> {
        self.db.execute(
            "insert into programs (program, protected) values (?1, ?2)
             on conflict (program) do update set protected = ?2",
            params![program, protected],
        )?;
        Ok(())
    }

    pub(super) fn size(&self) -> Result<u64, StoreError> {
        let total = self
            .db
            .query_row("select coalesce(sum(size), 0) from objects", [], |row| {
                row.get(0)
            })?;
        Ok(total)
    }

    pub(super) fn loose(&self) -> Result<Vec<(String, String, u64)>, StoreError> {
        let mut query = self.db.prepare(
            "select u.url, o.hash, o.size from urls u join objects o on o.hash = u.hash
             where not exists (
                 select 1 from holders h join programs p on p.program = h.program
                 where h.url = u.url and p.protected = 1
             )
             order by o.used_at asc, u.url asc",
        )?;
        let rows = query.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub(super) fn holds(&self, hash: &str) -> Result<bool, StoreError> {
        let held = self.db.query_row(
            "select exists (select 1 from urls where hash = ?1)",
            params![hash],
            |row| row.get(0),
        )?;
        Ok(held)
    }

    pub(super) fn forget(&mut self, url: &str, hash: &str) -> Result<(), StoreError> {
        let write = self.db.transaction()?;
        write.execute("delete from urls where url = ?1", params![url])?;
        write.execute("delete from holders where url = ?1", params![url])?;
        write.execute(
            "delete from objects where hash = ?1
             and not exists (select 1 from urls where hash = ?1)",
            params![hash],
        )?;
        write.commit()?;
        Ok(())
    }
}
