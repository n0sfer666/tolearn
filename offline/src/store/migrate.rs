use rusqlite::{Connection, params};

use super::error::StoreError;

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
    last_modified text,
    path text,
    body_hash text,
    checked_at integer
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

const ADDED: [(&str, &str); 3] = [
    ("path", "text"),
    ("body_hash", "text"),
    ("checked_at", "integer"),
];

pub(super) fn setup(db: &Connection) -> Result<(), StoreError> {
    db.execute_batch(SCHEMA)?;
    widen(db)
}

fn widen(db: &Connection) -> Result<(), StoreError> {
    for (column, kind) in ADDED {
        if has(db, column)? {
            continue;
        }
        let added = db.execute(&format!("alter table urls add column {column} {kind}"), []);
        if added.is_err() && !has(db, column)? {
            added?;
        }
    }
    Ok(())
}

fn has(db: &Connection, column: &str) -> Result<bool, StoreError> {
    let found: i64 = db.query_row(
        "select count(*) from pragma_table_info('urls') where name = ?1",
        params![column],
        |row| row.get(0),
    )?;
    Ok(found > 0)
}
