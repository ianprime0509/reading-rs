//! Provides functions for working with plan files stored in
//! a special directory (including a function to find and create
//! this directory).
//!
//! All plan files should be stored in the plans directory with
//! the extension `.plan.json`.

use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::fs::{self, File, ReadDir};
use std::io;
use std::iter::Iterator;
use std::path::PathBuf;

use serde_json;

use Plan;

/// Represents an error when working with plan files.
#[derive(Debug)]
pub enum Error {
    /// The user's config directory could not be found or deduced.
    CannotLocateConfig,
    /// The config directory does not exist (has not been created yet).
    NoConfigDirectory,
    /// The specified plan does not exist (includes the name of the plan).
    NoSuchPlan(String),
    /// The specified plan already exists (includes the name of the plan).
    /// This may be an error if, for example, the user tries to add
    /// a plan with the same name as one previously existing.
    PlanAlreadyExists(String),
    /// An error occurred when interacting with the filesystem or
    /// when reading or writing from/to a file.
    Io(io::Error),
    /// An error occurred when trying to serialize or deserialize
    /// JSON data.
    Json(serde_json::Error),
}

use self::Error::*;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NoSuchPlan(ref s) => write!(f, "Plan '{}' does not exist", s),
            PlanAlreadyExists(ref s) => write!(f, "Plan '{}' already exists", s),
            Io(ref e) => write!(f, "{}", e),
            Json(ref e) => write!(f, "{}", e),
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            CannotLocateConfig => "Could not find appropriate plan storage directory",
            NoConfigDirectory => {
                "Plan storage directory does not exist yet (you need to add some plans first)"
            }
            NoSuchPlan(_) => "No such plan exists",
            PlanAlreadyExists(_) => "Plan already exists",
            Io(ref e) => e.description(),
            Json(ref e) => e.description(),
        }
    }
}

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
    type Item = Result<Plan, Error>;

    fn next(&mut self) -> Option<Result<Plan, Error>> {
        let entry = match self.read_dir.next() {
            Some(e) => e,
            None => return None,
        };
        let path = match entry {
            Ok(p) => p.path(),
            Err(e) => return Some(Err(Io(e))),
        };

        // Make sure we skip over things that aren't files or
        // don't have the proper extension ('.plan.json')
        if !path.is_file() ||
           !path.to_str().expect("Path is not valid UTF8").ends_with(".plan.json") {
            return self.next();
        }
        // Now try to open the plan and read in its data
        let f = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Some(Err(Io(e))),
        };
        match serde_json::from_reader(&f) {
            Ok(p) => Some(Ok(p)),
            Err(e) => Some(Err(Json(e))),
        }
    }
}

/// Returns an iterator over the plans in the plan directory if possible,
/// or an error if this cannot be done.
pub fn plans() -> Result<Plans, Error> {
    let dir = plans_dir_must_exist()?;

    Ok(Plans { read_dir: fs::read_dir(&dir).map_err(|e| Io(e))? })
}

/// Returns the location of the plans directory if possible.
pub fn plans_dir() -> Result<PathBuf, Error> {
    match env::home_dir() {
        Some(mut d) => {
            d.push(".reading");
            Ok(d)
        }
        None => Err(CannotLocateConfig),
    }
}

/// Returns the location of the plans directory, ensuring that it
/// actually exists (the directory will be created if it does not).
fn plans_dir_ensure() -> Result<PathBuf, Error> {
    let path = plans_dir()?;
    if !path.exists() || !path.is_dir() {
        fs::create_dir(&path).map(|_| path).map_err(|e| Io(e))
    } else {
        Ok(path)
    }
}

/// Returns the location of the plans directory, returning an error
/// if it doesn't exist.
fn plans_dir_must_exist() -> Result<PathBuf, Error> {
    let path = plans_dir()?;
    if !path.exists() || !path.is_dir() {
        Err(NoConfigDirectory)
    } else {
        Ok(path)
    }
}

/// Reads the plan with the given name
pub fn read_plan(name: &str) -> Result<Plan, Error> {
    let mut filename = plans_dir_must_exist()?;
    filename.push(name);
    filename.set_extension("plan.json");

    if !filename.exists() {
        return Err(NoSuchPlan(name.into()));
    }
    // We need to map the error into the correct type (wrap it in the
    // Io variant of our custom error)
    let f = File::open(filename).map_err(|e| Io(e))?;

    serde_json::from_reader(f).map_err(|e| Json(e))
}

/// Writes the given plan to the plans directory, and will return
/// an error if the plan already exists there.
pub fn add_plan(p: &Plan) -> Result<(), Error> {
    let mut filename = plans_dir_ensure()?;
    filename.push(p.name());
    filename.set_extension("plan.json");

    if filename.exists() {
        return Err(PlanAlreadyExists(p.name().into()));
    }
    let mut f = File::create(filename).map_err(|e| Io(e))?;

    serde_json::to_writer(&mut f, &p).map_err(|e| Json(e))
}

/// Writes the given plan to the plans directory, overwriting it if
/// it already exists.
pub fn overwrite_plan(p: &Plan) -> Result<(), Error> {
    let mut filename = plans_dir_ensure()?;
    filename.push(p.name());
    filename.set_extension("plan.json");

    let mut f = File::create(filename).map_err(|e| Io(e))?;

    serde_json::to_writer(&mut f, &p).map_err(|e| Json(e))
}

/// Attempts to remove the plan with the given name, returning
/// an error if it doesn't exist.
pub fn remove_plan(name: &str) -> Result<(), Error> {
    let mut filename = plans_dir_must_exist()?;
    filename.push(name);
    filename.set_extension("plan.json");

    if !filename.exists() {
        Err(NoSuchPlan(name.to_owned()))
    } else {
        fs::remove_file(&filename).map_err(|e| Io(e))
    }
}
