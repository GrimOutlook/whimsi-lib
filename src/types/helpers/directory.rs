use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use ambassador::Delegate;
use anyhow::ensure;
use derive_more::{Display, From};
use getset::Getters;
use itertools::Itertools;
use thiserror::Error;

use crate::types::column::identifier::Identifier;
use crate::types::properties::systemfolder::SystemFolder;

use super::filename::Filename;
use super::node::Node;

// TODO: If the `getset` crate ever supports Traits, use them here. I should not have to manually
// make getters just because they are contained in traits.
#[ambassador::delegatable_trait]
pub trait DirectoryKind: Clone {
    fn contained(&self) -> Vec<Node>;
    fn contained_mut(&mut self) -> &mut Vec<Node>;

    fn contained_directories(&self) -> Vec<Rc<RefCell<SubDirectory>>> {
        self.contained()
            .iter()
            .filter_map(|node| node.try_as_directory_ref())
            .cloned()
            .collect_vec()
    }

    fn insert_dir(&mut self, name: &str) -> anyhow::Result<Rc<RefCell<SubDirectory>>> {
        let new_filename = Filename::parse(name)?;
        self.insert_dir_filename(new_filename)
    }

    fn insert_dir_with_trim(&mut self, name: &str) -> anyhow::Result<Rc<RefCell<SubDirectory>>> {
        let new_filename = Filename::parse_with_trim(name)?;
        self.insert_dir_filename(new_filename)
    }

    fn insert_dir_filename(
        &mut self,
        filename: Filename,
    ) -> anyhow::Result<Rc<RefCell<SubDirectory>>> {
        ensure!(
            !self
                .contained()
                .iter()
                .filter_map(|node| node.try_as_directory_ref())
                .any(|dir| dir.borrow().name == filename),
            DirectoryConversionError::DuplicateDirectoryName
        );

        let wrapped_new_dir = SubDirectory::new(filename);
        let new_dir = Rc::new(RefCell::new(wrapped_new_dir));
        self.contained_mut().push(new_dir.clone().into());
        Ok(new_dir)
    }
}
macro_rules! implement_directory_kind_simple {
    ($struct_name:ty) => {
        impl DirectoryKind for $struct_name {
            fn contained(&self) -> Vec<Node> {
                self.contained.clone()
            }

            fn contained_mut(&mut self) -> &mut Vec<Node> {
                &mut self.contained
            }
        }
    };
}

#[derive(Clone, Debug, Display, PartialEq, Getters)]
#[display("{}", id)]
#[getset(get = "pub")]
pub struct SystemDirectory {
    #[getset(skip)]
    contained: Vec<Node>,

    /// ID of this directory.
    id: SystemFolder,
    /// Name for the system directory. This is automatically derived during installation if `.` is
    /// used.
    name: Filename,
}

implement_directory_kind_simple!(SystemDirectory);

/// Directory that is a contained within a subdirectory.
///
/// The ID for this directory is created upon insertion into the tables database.
#[derive(Clone, Debug, Display, PartialEq, Getters)]
#[display("{}", name)]
#[getset(get = "pub")]
pub struct SubDirectory {
    #[getset(skip)]
    contained: Vec<Node>,

    id: Option<Identifier>,
    /// The directory's name (localizable)
    name: Filename,
}

impl SubDirectory {
    pub fn new(name: Filename) -> Self {
        Self {
            contained: Vec::new(),
            id: None,
            name,
        }
    }
}

implement_directory_kind_simple!(SubDirectory);

impl FromStr for SubDirectory {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(Filename::parse(s)?))
    }
}

#[derive(Clone, Debug, Delegate, Display, From, PartialEq, strum::EnumIs)]
#[delegate(DirectoryKind)]
pub enum Directory {
    SystemDirectory(SystemDirectory),
    SubDirectory(SubDirectory),
}

impl Directory {
    /// Create a new system directory under root.
    pub fn system_directory(system_folder: SystemFolder) -> SystemDirectory {
        SystemDirectory {
            contained: Vec::new(),
            id: system_folder.clone(),
            name: Filename::parse(".").unwrap().clone(),
        }
    }
}

#[derive(Debug, Error, From)]
pub enum DirectoryConversionError {
    #[error("Given directory name cannot fit in short filename")]
    DirectoryNameTooLong,
    #[error("Directory name already exists in parent directory")]
    DuplicateDirectoryName,
}

#[cfg(test)]
mod test {
    use assertables::assert_contains;

    use crate::types::{helpers::directory::DirectoryKind, properties::systemfolder::SystemFolder};

    use super::Directory;

    #[test]
    fn add_directory() {
        let mut pf = Directory::system_directory(SystemFolder::ProgramFiles);
        let man = pf.insert_dir("MAN").unwrap();
        assert_contains!(pf.contained(), &man.clone().into());
        assert_eq!(man.borrow().name().to_string(), "MAN");
    }
}
