use super::{Plan, Entry};

use std::error::Error as StdError;
use std::fmt;
use std::io::{self, Read, BufRead, BufReader};

/// Represents an error in working with a `Plan` (e.g. if a nonexistent
/// entry is accessed, or if there is an IO error in reading a plan
/// from a file).
#[derive(Debug)]
pub enum Error {
    /// An IO error was encountered while processing. Contains the
    /// base `io::Error` that describes the problem.
    Io(io::Error),
    /// There was an error in the text input format (e.g. the given
    /// input was empty).
    TextFormat(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TextFormat(ref s) => write!(f, "{}", s),
            Io(ref e) => write!(f, "{}", e),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            TextFormat(_) => "Error in text input format",
            Io(ref e) => e.description(),
        }
    }
}

use self::Error::*;

impl Entry {
    /// Returns an `Entry` with a title and no description.
    pub fn new(title: &str) -> Entry {
        Entry::with_description(title, "")
    }

    /// Returns an `Entry` with the given title and description.
    pub fn with_description(title: &str, description: &str) -> Entry {
        Entry {
            title: title.into(),
            description: description.into(),
        }
    }

    /// Returns the title of the entry.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the description of the entry.
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl Plan {
    /// Attempts to construct a plan from plain text input.
    ///
    /// The expected format of a plan in plain text is a series of
    /// unindented lines, representing the titles of the entries in the
    /// plan, each of which may be followed by indented lines containing
    /// a more detailed description of that entry.
    /// Note that any amount of indentation (tabs and/or spaces) will
    /// be considered as a description, and that a blank line will terminate
    /// any entry.
    pub fn from_text<T: Read>(name: &str, input: T) -> Result<Plan, Error> {
        // Buffer the reader so that we can read by lines
        let r = BufReader::new(input);
        let mut entries = Vec::new();
        // The current entry being processed
        let mut current_entry: Option<Entry> = None;

        for (n, l) in r.lines().enumerate() {
            // Check for any IO errors in reading the line
            // Also, trim any whitespace to the right of the line,
            // since it doesn't matter.
            let l = match l {
                Ok(l) => l.trim_right().to_owned(),
                Err(e) => return Err(Io(e)),
            };
            // Skip blank lines, but consider them to be the end of an entry if present
            if l.is_empty() {
                if let Some(e) = current_entry {
                    entries.push(e);
                    current_entry = None;
                }
                continue;
            }

            // Check to see if this is part of the description by
            // looking for indentation
            if l.chars().nth(0).unwrap().is_whitespace() {
                // Add to the description of the current entry
                match &mut current_entry {
                    &mut Some(ref mut e) => {
                        // Add a space to the description before adding a
                        // new line of it
                        if !e.description.is_empty() {
                            e.description += " ";
                        }
                        e.description += l.trim_left();
                    }
                    &mut None => {
                        return Err(TextFormat(format!("Description on line {} does not \
                                                       correspond to any entry",
                                                      n + 1)));
                    }
                }
            } else {
                // This is the title of a new entry, so add the previous
                // entry to the list and start a new one
                if let Some(e) = current_entry {
                    entries.push(e);
                }

                current_entry = Some(Entry::new(&l));
            }
        }

        // Add any entry that is left at the end
        if let Some(e) = current_entry {
            entries.push(e);
        }

        if entries.is_empty() {
            Err(TextFormat("Cannot construct an empty plan".into()))
        } else {
            Ok(Plan {
                name: name.into(),
                cyclic: false,
                current_entry: 0,
                entries: entries,
            })
        }
    }

    /// Advances the plan by the given number of entries.
    ///
    /// For a cyclic plan, this will wrap around; for an acyclic plan,
    /// this will either go to the beginning of the plan or to
    /// the "end of plan" position as appropriate.
    ///
    /// A negative increment can be specified.
    pub fn next(&mut self, inc: i32) {
        let mut new_entry = self.current_entry as i32 + inc;
        let n_entries = self.entries.len() as i32;

        // Adjust out of range entries as appropriate for cyclic/acyclic plans
        if new_entry < 0 {
            if self.cyclic {
                new_entry = new_entry % n_entries as i32 + n_entries as i32;
            } else {
                new_entry = 0;
            }
        } else if new_entry >= n_entries {
            if self.cyclic {
                new_entry %= n_entries;
            } else {
                new_entry = n_entries;
            }
        }

        // Set the current entry
        self.current_entry = new_entry as usize;
    }

    /// Reverts the plan by the given number of entries.
    ///
    /// This is simply a shortcut for using `next` with a negative
    /// increment.
    pub fn previous(&mut self, dec: i32) {
        self.next(-dec)
    }

    /// Returns the name of the plan.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the current entry number of the plan (as a 1-based index).
    pub fn current_entry_number(&self) -> usize {
        self.current_entry + 1
    }

    /// Returns the number of entries in the plan.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns the current `Entry` of the plan, or `None` if we are
    /// at the end of the plan.
    pub fn current_entry(&self) -> Option<&Entry> {
        self.entries.get(self.current_entry)
    }
}
