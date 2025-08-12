use anyhow::bail;
use itertools::Itertools;
use thiserror::Error;

use crate::Msi;
use crate::types::dao::directory::DirectoryDao;
use crate::types::helpers::directory::{Directory, DirectoryKind, SubDirectory, SystemDirectory};

use super::TableKind;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DirectoryTable(Vec<DirectoryDao>);

impl Msi {
    pub fn add_directory_recursive(
        &mut self,
        directory: &SubDirectory,
        parent: Directory,
    ) -> anyhow::Result<()> {
        // Check if the parent is not in the identities hashmap and if it is not a system folder
        // return an error.
        let parent_id = if let Directory::SubDirectory(parent_dir) = parent {
            let parent_id = parent_dir.id();
            if !self.identifiers.keys().contains(parent_id) {
                bail!(MsiError::NoParentForSubdirectory)
            } else {
                parent_id
            };
        } else {
        };
        // Create the table if it doesn't exist
        let mut table: DirectoryTable = self.table_or_new(TableKind::Directories).try_into()?;
        let new_dao = DirectoryDao::new(directory, &parent)?;
        table.0.push(new_dao);
        self.add_children(directory)?;
        Ok(())
    }

    // Root is the only directory that doesn't require a parent
    fn add(&mut self, system_dir: SystemDirectory) -> anyhow::Result<()> {
        let mut table: DirectoryTable = self.table(TableKind::Directories).unwrap().try_into()?;
        table.0.push((&system_dir).try_into()?);
        self.add_children(&system_dir)?;
        Ok(())
    }

    fn add_children(&mut self, directory: &impl DirectoryKind) -> anyhow::Result<()> {
        for child in directory.contained_directories() {
            self.add_directory_recursive(&child.borrow(), directory)?;
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum MsiError {
    #[error("Subdirectory's parent ID not found in already defined identities")]
    NoParentForSubdirectory,
}
