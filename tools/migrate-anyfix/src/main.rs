//! Converts section-syntax documents to anyfix and checks each conversion.
//!
//! Reads a JSON array of `{ id, name, content, revision }` on stdin and writes
//! one `FileMigration` per row to stdout. Never touches a database:
//! `scripts/migrate-anyfix-d1.py` fetches the rows and applies the result.

#![cfg_attr(test, allow(clippy::disallowed_macros))]

mod migration;
#[cfg(test)]
mod tests;

use migration::{migrate, FileRow};

fn main() {
    let rows: Vec<FileRow> = serde_json::from_reader(std::io::stdin().lock())
        .expect("stdin should be a JSON array of file rows");
    let migrations: Vec<_> = rows.iter().map(migrate).collect();
    serde_json::to_writer(std::io::stdout().lock(), &migrations)
        .expect("stdout should be writable");
}
