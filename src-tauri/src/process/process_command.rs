use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use shell_words::ParseError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProcessCommand {
    pub command: PathBuf,
    pub args: Vec<String>,
}

impl ProcessCommand {
    pub fn new<P: Into<PathBuf>>(command: P, args: Vec<String>) -> Self {
        ProcessCommand {
            command: command.into(),
            args,
        }
    }
    pub fn set_command<S: Into<PathBuf>>(&mut self, command: S) {
        self.command = command.into();
    }
    pub fn set_args(&mut self, args: Vec<String>) {
        self.args = args;
    }
    pub fn get_command(&self, install_dir: &PathBuf) -> PathBuf {
        install_dir.join(self.command.clone())
    }

    // Returns true if something was set, false if not
    pub fn set<S: Into<String>>(&mut self, s: S) -> Result<bool, ParseError> {
        let s: String = s.into();
        let binding = shell_words::split(&s).map(|s| s.to_owned())?;
        let parsed = binding.split_first();
        match parsed {
            Some(command) => {
                (self.command, self.args) = (PathBuf::from(command.0), command.1.to_vec())
            }
            None => return Ok(false),
        };
        return Ok(true);
    }
    pub fn into_readable(&self) -> (PathBuf, Vec<String>) {
        (PathBuf::from(self.command.clone()), self.args.clone())
    }
    pub fn into_readable_with_dir(&self, install_dir: &PathBuf) -> (PathBuf, Vec<String>) {
        (self.get_command(install_dir), self.args.clone())
    }
    fn process_command<P: AsRef<Path>>(install_dir: &PathBuf, command: P) -> PathBuf {

        let install_dir = Path::new(install_dir);
        let absolute_exe = install_dir.join(command);

        absolute_exe
    }
}
