// This is basically an internal representation of what is contained in the final MSI, just in more
// manipulatable data types.
//
// Properties are derived from this table:
// https://learn.microsoft.com/en-us/windows/win32/msi/database-tables
// This is basically an internal representation of what is contained in the final MSI, just in more
// manipulatable data types.
//
// Properties are derived from this table:
// https://learn.microsoft.com/en-us/windows/win32/msi/database-tables
// TODO: Figure out why this causes tests to not run.
// TODO: Figure out why this causes the rust-lsp to break and tests to not run.
// #![cfg(not(debug_assertions))]
// #![deny(
//     clippy::all,
//     missing_docs,
//     missing_debug_implementations,
//     rustdoc::all,
//     unsafe_code
// )]
#![cfg(debug_assertions)]
#![allow(dead_code)]

pub mod constants;
mod tables;
pub mod types;

use std::collections::HashMap;

use getset::Getters;
use strum::IntoDiscriminant;
use tables::{Table, TableKind};
use types::column::{ColumnValue, identifier::Identifier};

type Identifiers = HashMap<Identifier, ColumnValue>;

/// An in-memory representation of the final MSI to be created.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct Msi {
    /// Tracks identifiers used to relate items between tables.
    identifiers: Identifiers,
    tables: Vec<Table>,
}

impl Msi {
    pub fn table(&self, table: TableKind) -> Option<Table> {
        self.tables
            .iter()
            .find(|t| t.discriminant() == table)
            .cloned()
    }

    pub fn table_or_new(&self, table: TableKind) -> Table {
        if let Some(t) = self.table(table) {
            return t;
        }

        todo!("Create new table")
    }
}
