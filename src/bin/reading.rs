extern crate reading;

extern crate ansi_term;
extern crate clap;

use std::fs::File;
use std::path::Path;

use ansi_term::{Colour, Style};
use clap::{Arg, ArgMatches, App, AppSettings, SubCommand};

use reading::{files, Plan};

/// Describes all the styles that can be used in printing text.
/// This can be used for custom themes eventually maybe?
/// Mostly just good for disabling custom formatting.
struct StyleSet {
    /// Normal text
    normal: Style,
    /// Title text
    title: Style,
    /// Description (or label) text
    description: Style,
    /// Error text
    error: Style,
}

impl StyleSet {
    /// Preset for the --no-ansi option (no style)
    fn no_ansi() -> StyleSet {
        StyleSet {
            normal: Style::new(),
            title: Style::new(),
            description: Style::new(),
            error: Style::new(),
        }
    }

    /// Preset for the normal "fancy" style
    fn fancy() -> StyleSet {
        StyleSet {
            normal: Style::new(),
            title: Colour::White.bold(),
            description: Style::new().italic(),
            error: Colour::Red.normal(),
        }
    }
}

pub fn main() {
    let matches = App::new("reading")
        .version("0.1.0")
        .author("Ian Johnson <ianprime0509@gmail.com>")
        .about("A simple reading plan manager")
        .setting(AppSettings::ColoredHelp)
        .arg(Arg::with_name("no-ansi")
            .help("Disables fancy text output")
            .short("n")
            .long("no-ansi"))
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
        .subcommand(SubCommand::with_name("remove")
            .about("Removes a reading plan from the collection")
            .arg(Arg::with_name("PLAN")
                .help("The name of the plan to remove")
                .required(true)))
        .subcommand(SubCommand::with_name("list").about("Lists all installed reading plans"))
        .subcommand(SubCommand::with_name("view")
            .about("Views the current entry (and optionally more) of the specified plan")
            .arg(Arg::with_name("PLAN")
                .help("The name of the plan to view")
                .required(true))
            .arg(Arg::with_name("count")
                .short("c")
                .long("count")
                .value_name("COUNT")
                .help("The number of following entries to view (default 1)")
                .takes_value(true)))
        .after_help("Longer explanation goes here")
        .get_matches();

    // Whether we should disable the fancy ANSI terminal text
    let no_ansi = matches.is_present("no-ansi");
    // The style to use
    let style_set = if no_ansi {
        StyleSet::no_ansi()
    } else {
        StyleSet::fancy()
    };

    // Run the appropriate subcommand
    match matches.subcommand() {
        ("add", Some(sub_m)) => add(sub_m, style_set),
        ("remove", Some(sub_m)) => remove(sub_m, style_set),
        ("list", Some(sub_m)) => list(sub_m, style_set),
        ("view", Some(sub_m)) => view(sub_m, style_set),
        _ => println!("{}", Colour::White.bold().paint("Test text")),

    }
}

/// The `add` subcommand logic
fn add(m: &ArgMatches, style_set: StyleSet) {
    let filename = Path::new(m.value_of("FILENAME").unwrap());
    // Get the name of the plan; either provided explicitly or
    // deduced from the file name
    let name = m.value_of("name").unwrap_or(match filename.file_stem() {
        Some(n) => n.to_str().expect("Invalid UTF8 in filename"),
        None => {
            println!("{}",
                     style_set.error
                         .paint(format!("Could not deduce plan name from filename '{}'",
                                        filename.display())));
            return;
        }
    });

    // Try to open the file and parse a plan from it
    let f = match File::open(&filename) {
        Ok(f) => f,
        Err(e) => {
            println!("{}",
                     style_set.error
                         .paint(format!("Error opening file {}: {}", filename.display(), e)));
            return;
        }
    };
    let plan = match Plan::from_text(name, &f) {
        Ok(p) => p,
        Err(e) => {
            println!("{}",
                     style_set.error.paint(format!("Error parsing plan: {}", e)));
            return;
        }
    };

    // Now add the plan to the plans directory
    if let Err(e) = files::add_plan(&plan) {
        println!("{}",
                 style_set.error.paint(format!("Error adding plan: {}", e)));
    } else {
        println!("{}", style_set.normal.paint(format!("Added plan {}", name)));
    }
}

/// The `remove` subcommand logic
fn remove(m: &ArgMatches, style_set: StyleSet) {
    let name = m.value_of("PLAN").unwrap();

    if let Err(e) = files::remove_plan(name) {
        println!("{}",
                 style_set.error.paint(format!("Error removing plan: {}", e)));
    } else {
        println!("{}",
                 style_set.normal.paint(format!("Removed plan {}", name)));
    }
}

/// The `list` subcommand logic
fn list(_m: &ArgMatches, style_set: StyleSet) {
    let plans = match files::plans() {
        Ok(p) => p,
        Err(e) => {
            println!("{}",
                     style_set.error.paint(format!("Could not open plans folder: {}", e)));
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
        // Check for end of plan (current == len indicates this)
        if current == len {
            println!("{} {}",
                     style_set.title.paint(name),
                     style_set.normal.paint("(end of plan)"));
        } else {
            println!("{} {}",
                     style_set.title.paint(name),
                     style_set.normal.paint(format!("(entry {} of {})", current, len)));
        }
    }

    // Output any failures
    match failures {
        0 => {}
        1 => println!("{}", style_set.error.paint("1 plan could not be read")),
        n @ _ => {
            println!("{}",
                     style_set.error.paint(format!("{} plans could not be read", n)))
        }
    }
}

/// The `view` subcommand logic
fn view(m: &ArgMatches, style_set: StyleSet) {
    let name = m.value_of("PLAN").unwrap();
    let count = match m.value_of("count") {
        None => 1,
        Some(num) => {
            match num.parse() {
                Ok(n) => n,
                Err(_) => {
                    println!("{}",
                             style_set.error.paint("Invalid numeric argument to `--count`"));
                    return;
                }
            }
        }
    };

    let plan = match files::read_plan(name) {
        Ok(p) => p,
        Err(e) => {
            println!("{}",
                     style_set.error.paint(format!("Error reading plan: {}", e)));
            return;
        }
    };

    // Print out the given number of entries, starting at the current one
    for (n, entry) in plan.entries().skip(plan.current_entry_number() - 1).take(count).enumerate() {
        let label = match n {
            0 => "Current entry: ".to_owned(),
            1 => "Next entry: ".to_owned(),
            _ => format!("{} entries from now: ", n),
        };

        println!("{} {}",
                 style_set.normal.paint(format!("{:20}", label)),
                 style_set.title.paint(entry.title()));
        if !entry.description().is_empty() {
            println!("{:20} {}",
                     "",
                     style_set.description.paint(entry.description()));
        }
    }
}
