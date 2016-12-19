// For `error_chain!`
#![recursion_limit = "1024"]

#![cfg_attr(feature = "serde_derive", feature(proc_macro))]

#[cfg(feature = "serde_derive")]
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

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
    #[test]
    fn it_works() {}
}
