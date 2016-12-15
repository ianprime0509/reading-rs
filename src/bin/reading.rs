extern crate reading;

extern crate clap;

use std::fs::File;
use std::path::Path;

use clap::{Arg, ArgMatches, App, SubCommand};

use reading::{files, Plan};

pub fn main() {
    let matches = App::new("reading")
        .version("0.1.0")
        .author("Ian J. <ianprime0509@gmail.com>")
        .about("A simple reading plan manager")
        .subcommand(SubCommand::with_name("add")
            .about("Adds a reading plan to the collection")
            .arg(Arg::with_name("FILENAME")
                .help("The filename of the plan to add")
                .required(true))
            .arg(Arg::with_name("name")
                .short("n")
                .long("name")
                .value_name("NAME")
                .help("The name of the plan after adding")
                .takes_value(true))
            .arg(Arg::with_name("cyclic")
                .short("c")
                .long("cyclic")
                .help("Create a cyclic plan")))
        .subcommand(SubCommand::with_name("list").about("Lists all installed reading plans"))
        .after_help("Longer explanation goes here")
        .get_matches();

    match matches.subcommand() {
        ("add", Some(sub_m)) => add(sub_m),
        ("list", Some(sub_m)) => list(sub_m),
        _ => println!("Default action"),

    }
}

/// The `add` subcommand logic
fn add(m: &ArgMatches) {
    let filename = Path::new(m.value_of("FILENAME").unwrap());
    // Get the name of the plan; either provided explicitly or
    // deduced from the file name
    let name = m.value_of("name").unwrap_or(match filename.file_stem() {
        Some(n) => n.to_str().expect("Invalid UTF8 in filename"),
        None => {
            println!("Could not deduce plan name from filename '{}'",
                     filename.display());
            return;
        }
    });

    // Try to open the file and parse a plan from it
    let f = match File::open(&filename) {
        Ok(f) => f,
        Err(e) => {
            println!("Error opening file {}: {}", filename.display(), e);
            return;
        }
    };
    let plan = match Plan::from_text(name, &f) {
        Ok(p) => p,
        Err(e) => {
            println!("Error parsing plan: {}", e);
            return;
        }
    };

    // Now add the plan to the plans directory
    if let Err(e) = files::add_plan(&plan) {
        println!("Error adding plan: {}", e);
    }
}

/// The `list` subcommand logic
fn list(_m: &ArgMatches) {
    let plans = match files::plans() {
        Ok(p) => p,
        Err(e) => {
            println!("Could not open plans folder: {}", e);
            return;
        }
    };

    // Contains the name of the plan, the current entry number,
    // and the total number of entries
    let mut plan_list = Vec::new();
    // Keeps track of how many read failures we've had
    let mut failures = 0;

    for plan in plans {
        match plan {
            Ok(p) => plan_list.push((p.name().to_owned(), p.current_entry_number(), p.len())),
            Err(_) => failures += 1,
        }
    }

    // Now print out all the data
    for (name, current, len) in plan_list {
        println!("{}: (entry {} of {})", name, current, len);
    }

    // Output any failures
    match failures {
        0 => {}
        1 => println!("1 plan could not be read"),
        n @ _ => println!("{} plans could not be read", n),
    }
}
