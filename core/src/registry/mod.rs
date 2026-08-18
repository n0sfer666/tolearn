mod error;
mod read;
mod types;
mod write;

pub use error::RegistryError;
pub use types::{Listed, Program};

use std::io::ErrorKind;
use std::path::Path;

use crate::atomic;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Registry {
    programs: Vec<Program>,
}

impl Registry {
    pub fn read(path: &Path) -> Result<Self, RegistryError> {
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(RegistryError::Unreadable(error)),
        };
        let programs = read::programs(&source).map_err(RegistryError::Malformed)?;
        Ok(Self { programs })
    }

    pub fn save(&self, path: &Path) -> Result<(), RegistryError> {
        atomic::write(path, &write::text(&self.programs)).map_err(RegistryError::Unwritable)
    }

    pub fn programs(&self) -> &[Program] {
        &self.programs
    }

    pub fn entries(&self) -> Vec<Listed> {
        self.programs
            .iter()
            .map(|program| Listed {
                reachable: program.path.is_dir(),
                program: program.clone(),
            })
            .collect()
    }

    pub fn add(&mut self, program: Program) {
        match self.find(&program.id) {
            Some(at) => {
                let opened_at = self.programs[at].opened_at.clone();
                self.programs[at] = Program {
                    opened_at: program.opened_at.or(opened_at),
                    ..program
                };
            }
            None => self.programs.push(program),
        }
    }

    pub fn forget(&mut self, id: &str) {
        self.programs.retain(|program| program.id != id);
    }

    pub fn touch(&mut self, id: &str, at: &str) {
        if let Some(index) = self.find(id) {
            self.programs[index].opened_at = Some(at.to_owned());
        }
    }

    fn find(&self, id: &str) -> Option<usize> {
        self.programs.iter().position(|program| program.id == id)
    }
}
