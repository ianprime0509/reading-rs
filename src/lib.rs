//! When used as a library, `reading` provides all the features of the binary
//! program, exposed in a way that they can be reused by others (e.g. eventually
//! I might like to make a GUI interface without rewriting all this code).
//! The library is split into three modules: `errors`, which provides
//! all the error types (provided by `error_chain`); `plan`, which provides
//! the basic types for working with plans, such as `Plan`; and `files`,
//! which provides methods for working with plans stored in a system-dependent
//! configuration directory.
//!
//! More information on each module (except `errors`, which is self-explanatory)
//! is provided in the module-level documentation for each. Several fundamental
//! types,
//! such as `Plan` and `Error`, are re-exported as members of this module for
//! convenient use.

// For `error_chain!`
#![recursion_limit = "1024"]

#![cfg_attr(feature = "serde_derive", feature(proc_macro))]

#[cfg(feature = "serde_derive")]
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate app_dirs;
#[macro_use]
extern crate error_chain;

pub mod errors {
    error_chain!{
        errors {
            /// The user's config directory could not be found or deduced.
            CannotLocateConfig {
                description("cannot locate config directory")
            }
            /// The config directory does not exist (has not been created yet).
            NoConfigDirectory {
                description("config directory does not exist")
            }
            /// The specified plan does not exist (includes the name of the plan).
            PlanDoesNotExist(name: String) {
                description("plan does not exist")
                display("plan '{}' does not exist", name)
            }
            /// The specified plan already exists (includes the name of the plan).
            /// This may be an error if, for example, the user tries to add
            /// a plan with the same name as one previously existing.
            PlanAlreadyExists(name: String) {
                description("plan already exists")
                display("plan '{}' already exists", name)
            }
            /// Indicates an error in UTF8 format (probably a filename).
            Utf8(t: String) {
                description("utf8 error")
                    display("utf8 error: {}", t)
            }
            /// An error in plan text format.
            TextFormat(t: String) {
                description("text format error")
                display("text format error: {}", t)
            }
            /// An IO error (usually caused by `std::io::Error`).
            Io(t: String) {
                description("io error")
                display("{}", t)
            }
            /// A JSON error (usually caused by `serde_json::Error`).
            Json(t: String) {
                description("json error")
                display("{}", t)
            }
        }
    }
}

pub use errors::*;

pub mod plan;
pub mod files;

pub use plan::{Plan, Entry};

#[cfg(test)]
mod tests {
    use Plan;
    use Entry;

    #[test]
    fn plan_from_text() {
        let plan_text = "Entry 1
    Description
Entry 2
    Description

Entry 3";
        let plan = Plan::from_text("test", plan_text.as_bytes()).expect("could not parse plan");
        assert_eq!(plan.name(), "test");
        // Make sure the entries are as expected
        let entries: Vec<_> = plan.entries().collect();
        assert_eq!(plan.len(), 3);
        assert_eq!(entries[0],
                   &Entry::with_description("Entry 1", "Description"));
        assert_eq!(entries[1],
                   &Entry::with_description("Entry 2", "Description"));
        assert_eq!(entries[2], &Entry::new("Entry 3"));
    }

    #[test]
    fn plan_to_text() {
        let plan = Plan::from_entries("test",
                                      vec![Entry::with_description("Entry 1", "Desc 1"),
                                           Entry::new("Entry 2"),
                                           Entry::with_description("Entry 3", "Desc 3")]);
        let expected = "Entry 1
    Desc 1
Entry 2
Entry 3
    Desc 3\n";
        let mut buffer = Vec::new();

        plan.to_text(&mut buffer).expect("could not write to buffer");
        let buf_string = String::from_utf8(buffer).expect("did not write valid utf8");
        assert_eq!(buf_string.as_str(),
                   expected,
                   "expected: {}\ngot: {}",
                   expected,
                   buf_string);
    }

    #[test]
    fn cyclic() {
        let mut plan = Plan::from_entries("test", vec![Entry::new("entry"); 3]);
        plan.set_cyclic(true);
        assert!(plan.is_cyclic(), "plan is not cyclic");
        plan.next(3);
        assert_eq!(plan.current_entry_number(), 1);
        plan.previous(2);
        assert_eq!(plan.current_entry_number(), 2);
    }

    #[test]
    fn acyclic() {
        let mut plan = Plan::from_entries("test", vec![Entry::new("entry"); 3]);
        assert!(!plan.is_cyclic(), "plan is cyclic");
        plan.next(3);
        assert!(plan.is_ended(), "plan did not end");
        plan.previous(100);
        assert_eq!(plan.current_entry_number(), 1);
    }
}
