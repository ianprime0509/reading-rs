//! This module provides the basic `Plan` type and a variety of ways
//! to work with them, including reading and writing them from/to plain
//! text files, via the `from_text` and `to_text` methods, respectively.

use std::io::{Read, BufRead, BufReader, Write, BufWriter};
use std::slice;

// Bring in Serde types
#[cfg(feature = "serde_derive")]
include!("serde_types.in.rs");

#[cfg(feature = "serde_codegen")]
include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

use super::errors::*;

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
    /// Constructs a plan from a list of entries, setting the current entry
    /// to the first one. The resulting plan will be acyclic.
    pub fn from_entries(name: &str, entries: Vec<Entry>) -> Plan {
        Plan {
            name: name.to_owned(),
            cyclic: false,
            current_entry: 0,
            entries: entries,
        }
    }

    /// Attempts to construct a plan from plain text input.
    ///
    /// The expected format of a plan in plain text is a series of
    /// unindented lines, representing the titles of the entries in the
    /// plan, each of which may be followed by indented lines containing
    /// a more detailed description of that entry.
    /// Note that any amount of indentation (tabs and/or spaces) will
    /// be considered as a description, and that a blank line will terminate
    /// any entry.
    ///
    /// The resulting plan will be acyclic; this can be changed after creation
    /// with the `set_cyclic` method.
    pub fn from_text<T: Read>(name: &str, input: T) -> Result<Plan> {
        // Buffer the reader so that we can read by lines
        let r = BufReader::new(input);
        let mut entries = Vec::new();
        // The current entry being processed
        let mut current_entry: Option<Entry> = None;

        for (n, l) in r.lines().enumerate() {
            // Check for any IO errors in reading the line
            // Also, trim any whitespace to the right of the line,
            // since it doesn't matter.
            let l = l.chain_err(|| ErrorKind::Io("could not read line".into()))?
                .trim_right()
                .to_owned();
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
                        // So that rustfmt will work :P
                        return Err(ErrorKind::TextFormat(format!("description on line {} does \
                                                                  not correspond to any entry",
                                                                 n + 1))
                            .into());
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
            Err(ErrorKind::TextFormat("cannot construct an empty plan".into()).into())
        } else {
            Ok(Plan::from_entries(name, entries))
        }
    }

    /// Writes the plan using the standard plain text format to the specified writer.
    /// This format is documented in the documentation for `from_text`.
    pub fn to_text<T: Write>(&self, output: T) -> Result<()> {
        // Buffer writes
        let mut w = BufWriter::new(output);

        for e in self.entries() {
            writeln!(w, "{}", e.title()).chain_err(|| ErrorKind::Io("could not write to text output".into()))?;
            if !e.description().is_empty() {
                writeln!(w, "    {}", e.description()).chain_err(|| ErrorKind::Io("could not write to text output".into()))?;
            }
        }

        Ok(())
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

    /// Returns whether the plan is cyclic.
    pub fn is_cyclic(&self) -> bool {
        self.cyclic
    }

    /// Sets whether the plan is cyclic.
    ///
    /// If the plan is at its end when this is set, the current entry
    /// will be set to the first entry in the plan.
    pub fn set_cyclic(&mut self, cyclic: bool) {
        self.cyclic = cyclic;
        if self.cyclic && self.current_entry == self.len() {
            self.current_entry = 0;
        }
    }

    /// Returns the current entry number of the plan (as a 1-based index).
    /// If the plan is at its end, this will be 1 more than the length of
    /// the plan.
    pub fn current_entry_number(&self) -> usize {
        self.current_entry + 1
    }

    /// Returns the number of entries in the plan.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether this plan is at its end (for an acyclic plan).
    pub fn is_ended(&self) -> bool {
        self.current_entry_number() > self.len()
    }

    /// Returns the current `Entry` of the plan, or `None` if we are
    /// at the end of the plan.
    pub fn current_entry(&self) -> Option<&Entry> {
        self.entries.get(self.current_entry)
    }

    /// Returns an iterator over entries in the plan, of type `&Entry`
    pub fn entries(&self) -> slice::Iter<Entry> {
        self.entries.iter()
    }
}
