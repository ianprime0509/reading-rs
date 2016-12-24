//! This module provides functions for working with plan files stored in
//! a special directory (including a function to find and create
//! this directory). This directory is determined by the `app_dirs` crate,
//! which will return a path based on the operating system (Windows,
//! OS X, or Linux).
//!
//! All plan files should be stored in the plans directory with
//! the extension `.plan.json`. Files with a different extension
//! will not be recognized, e.g. by the `plans` iterator function.
//! In general, this should not be a problem; the provided methods
//! for adding/removing plans will provide this extension automatically.

use std::fs::{self, File, ReadDir};
use std::iter::Iterator;
use std::path::PathBuf;

use app_dirs::{self, AppInfo, AppDataType, AppDirsError};
use serde_json;

use super::Plan;
use super::errors::*;

/// The information for app_dirs
const APP_INFO: AppInfo = AppInfo {
    name: "reading",
    author: "Ian Johnson",
};

/// An iterator over all the plans in the plan directory.
///
/// The iterator returns items of type `Result<Plan, Error>`
/// because there may be errors in reading a plan or errors
/// in the format itself.
pub struct Plans {
    /// The underlying `ReadDir` iterator
    read_dir: ReadDir,
}

impl Iterator for Plans {
    type Item = Result<Plan>;

    fn next(&mut self) -> Option<Result<Plan>> {
        let entry = match self.read_dir.next() {
            Some(e) => e,
            None => return None,
        };
        let path = match entry.chain_err(|| ErrorKind::Io("could not read directory item".into())) {
            Ok(e) => e.path(),
            Err(e) => return Some(Err(e)),
        };

        // Make sure we skip over things that aren't files or
        // don't have the proper extension ('.plan.json')
        let path_str = match path.to_str() {
            Some(s) => s.to_owned(),
            None => return Some(Err(ErrorKind::Utf8("path is not valid utf8".into()).into())),
        };
        if !path.is_file() || !path_str.ends_with(".plan.json") {
            return self.next();
        }
        // Now try to open the plan and read in its data
        let f = match File::open(&path).chain_err(|| ErrorKind::Io(format!("could not open file '{}'", path.display()))) {
            Ok(f) => f,
            Err(e) => return Some(Err(e)),
        };
        Some(serde_json::from_reader(&f).chain_err(|| ErrorKind::Json(format!("json error in file '{}'", path.display()))))
    }
}

/// Returns an iterator over the plans in the plan directory if possible,
/// or an error if this cannot be done.
///
/// As noted in the module documentation, plans must have the extension
/// `.plan.json` to be recognized; the iterator will pass over any files
/// that do not have this extension.
pub fn plans() -> Result<Plans> {
    let dir = plans_dir_must_exist()?;

    Ok(Plans { read_dir: fs::read_dir(&dir).chain_err(|| ErrorKind::Io("could not read from plans directory".into()))? })
}

/// Returns the location of the plans directory if possible.
pub fn plans_dir() -> Result<PathBuf> {
    match app_dirs::get_app_dir(AppDataType::UserData, &APP_INFO, "plans") {
        Ok(p) => Ok(p),
        Err(AppDirsError::NotSupported) => Err(ErrorKind::CannotLocateConfig.into()),
        Err(AppDirsError::Io(e)) => Err(e).chain_err(|| ErrorKind::Io("could not find plans directory".into())),
        // This should properly be a panic, since there really isn't any way
        // this can happen (unless `app_dirs` changes in a breaking way).
        Err(AppDirsError::InvalidAppInfo) => panic!("invalid app info"),
    }
}

/// Returns the location of the plans directory, ensuring that it
/// actually exists (the directory will be created if it does not).
fn plans_dir_ensure() -> Result<PathBuf> {
    let path = plans_dir()?;
    if !path.exists() || !path.is_dir() {
        fs::create_dir(&path).map(|_| path).chain_err(|| ErrorKind::Io("could not create plans directory".into()))
    } else {
        Ok(path)
    }
}

/// Returns the location of the plans directory, returning an error
/// if it doesn't exist.
fn plans_dir_must_exist() -> Result<PathBuf> {
    let path = plans_dir()?;
    if !path.exists() || !path.is_dir() {
        Err(ErrorKind::NoConfigDirectory.into())
    } else {
        Ok(path)
    }
}

/// Reads the plan with the given name.
///
/// The filename of the plan must be `{name}.plan.json`, or it will
/// not be recognized.
pub fn read_plan(name: &str) -> Result<Plan> {
    let mut filename = plans_dir_must_exist()?;
    filename.push(name);
    filename.set_extension("plan.json");

    if !filename.exists() {
        return Err(ErrorKind::PlanDoesNotExist(name.into()).into());
    }
    // We need to map the error into the correct type (wrap it in the
    // Io variant of our custom error)
    let f = File::open(filename).chain_err(|| ErrorKind::Io("could not open plan file".into()))?;

    serde_json::from_reader(f).chain_err(|| ErrorKind::Json("json error in plan file".into()))
}

/// Writes the given plan to the plans directory, or will return
/// an error if the plan already exists there.
pub fn add_plan(p: &Plan) -> Result<()> {
    let mut filename = plans_dir_ensure()?;
    filename.push(p.name());
    filename.set_extension("plan.json");

    if filename.exists() {
        return Err(ErrorKind::PlanAlreadyExists(p.name().into()).into());
    }
    let mut f = File::create(filename).chain_err(|| ErrorKind::Io("could not create plan file".into()))?;

    serde_json::to_writer(&mut f, &p).chain_err(|| ErrorKind::Json("could not serialize plan to json".into()))
}

/// Writes the given plan to the plans directory, overwriting it if
/// it already exists.
pub fn overwrite_plan(p: &Plan) -> Result<()> {
    let mut filename = plans_dir_ensure()?;
    filename.push(p.name());
    filename.set_extension("plan.json");

    let mut f = File::create(filename).chain_err(|| ErrorKind::Io("could not overwrite plan file".into()))?;

    serde_json::to_writer(&mut f, &p).chain_err(|| ErrorKind::Json("could not serialize plan to json".into()))
}

/// Attempts to remove the plan with the given name, returning
/// an error if it doesn't exist.
pub fn remove_plan(name: &str) -> Result<()> {
    let mut filename = plans_dir_must_exist()?;
    filename.push(name);
    filename.set_extension("plan.json");

    if !filename.exists() {
        Err(ErrorKind::PlanDoesNotExist(name.to_owned()).into())
    } else {
        fs::remove_file(&filename).chain_err(|| ErrorKind::Io("could not remove plan file".into()))
    }
}
