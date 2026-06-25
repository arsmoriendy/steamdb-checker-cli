use super::prelude::*;
use std::fmt::Display;

use crate::entry::Entries;

#[derive(Default, Debug)]
pub struct State {
    pub csv_path: OsString,
    pub dir_path: OsString,

    pub entries: Entries,

    pub valid: Vec<OsString>,
    pub missing: Vec<OsString>,
    pub invalid: Vec<OsString>,

    pub validation_tx: Option<mpsc::Sender<OsString>>,

    pub extra: Vec<OsString>,
    pub extra_tx: Option<oneshot::Sender<bool>>,
}

fn print_files(
    kind: &str,
    files: &Vec<OsString>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    f.write_str(&format!("\n# {} {kind} file(s)\n\n", files.len()))?;
    for file in files {
        f.write_str(&format!("{file:?}\n"))?;
    }
    Ok(())
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        print_files("missing", &self.missing, f)?;
        print_files("invalid", &self.invalid, f)?;
        print_files("extra", &self.extra, f)?;

        Ok(())
    }
}
