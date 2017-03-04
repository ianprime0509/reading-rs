extern crate reading;

extern crate ansi_term;
extern crate clap;
#[macro_use]
extern crate error_chain;

use std::fs::File;
use std::path::Path;

use ansi_term::{Colour, Style};
use clap::{Arg, ArgMatches, App, AppSettings, SubCommand};

use reading::{files, Plan};
use reading::errors::*;

/// Describes all the styles that can be used in printing text.
/// This can be used for custom themes eventually maybe?
/// Mostly just good for disabling custom formatting.
#[derive(Debug, Clone)]
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

/// Returns styled text (using a format string syntax)
macro_rules! style {
    ($style:expr, $($arg:tt)*) => {
        {{
            $style.paint(format!( $($arg)*) )
        }}
    }
}

/// Prints a line of text in the given style
macro_rules! styleln {
    ($style:expr, $($arg:tt)*) => {
        println!("{}", style!($style, $($arg)*))
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
                .help("Create a cyclic plan"))
            .after_help("The expected input format is a plain text file, with each line \
                         representing the title of an entry in the plan. Optionally, a title  \
                         may be followed by a description, which is given on the line(s) \
                         directly following and marked as such by any level of indentation. If \
                         no name is provided for the plan, the filename (without the extension) \
                         will be used as the name."))
        .subcommand(SubCommand::with_name("remove")
            .about("Removes a reading plan from the collection")
            .arg(Arg::with_name("PLAN")
                .help("The name of the plan to remove")
                .required(true)))
        .subcommand(SubCommand::with_name("export")
            .about("Exports a reading plan to a plain text file")
            .arg(Arg::with_name("PLAN")
                .help("The name of the plan to export")
                .required(true))
            .arg(Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("OUTPUT")
                .help("The output filename")
                .takes_value(true))
            .after_help("If no output filename is specified, the filename will be '(name of \
                         plan) + .plan'."))
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
                .default_value("1")
                .help("The number of following entries to view")
                .takes_value(true)))
        .subcommand(SubCommand::with_name("next")
            .about("Moves the specified plan to the next entry")
            .arg(Arg::with_name("PLAN")
                .help("The plan to change")
                .required(true))
            .arg(Arg::with_name("count")
                .short("c")
                .long("count")
                .value_name("COUNT")
                .default_value("1")
                .help("The number of entries to move forward")
                .takes_value(true)))
        .subcommand(SubCommand::with_name("previous")
            .about("Moves the specified plan to the previous entry")
            .arg(Arg::with_name("PLAN")
                .help("The plan to change")
                .required(true))
            .arg(Arg::with_name("count")
                .short("c")
                .long("count")
                .value_name("COUNT")
                .default_value("1")
                .help("The number of entries to move backward")
                .takes_value(true)))
        .after_help("reading is a reading plan manager, but can also be used to manage other \
                     sorts of schedules or plans. To get started, use `reading add` to add a \
                     plan, and check `reading help add` for the expected input format.")
        .get_matches();

    // Whether we should disable the fancy ANSI terminal text
    let no_ansi = matches.is_present("no-ansi");
    // The style to use
    let style_set = if no_ansi {
        StyleSet::no_ansi()
    } else {
        StyleSet::fancy()
    };

    // Handle errors nicely
    if let Err(ref e) = run(matches, &style_set) {
        styleln!(style_set.error, "Error: {}", e);

        for e in e.iter().skip(1) {
            styleln!(style_set.error, "Caused by: {}", e);
        }

        if let Some(backtrace) = e.backtrace() {
            styleln!(style_set.error, "Backtrace: {:?}", backtrace);
        }

        std::process::exit(1);
    }
}

/// The main program logic.
/// Each subcommand should do its own printing, except for errors, which are returned.
fn run(m: ArgMatches, style_set: &StyleSet) -> Result<()> {
    // Run the appropriate subcommand
    match m.subcommand() {
        ("add", Some(sub_m)) => add(sub_m, style_set),
        ("remove", Some(sub_m)) => remove(sub_m, style_set),
        ("export", Some(sub_m)) => export(sub_m, style_set),
        ("list", Some(_)) => list(style_set),
        ("view", Some(sub_m)) => view(sub_m, style_set),
        ("next", Some(sub_m)) => next(sub_m, style_set, true),
        ("previous", Some(sub_m)) => next(sub_m, style_set, false),
        _ => list(style_set),
    }
}

/// The `add` subcommand logic.
fn add(m: &ArgMatches, style_set: &StyleSet) -> Result<()> {
    let filename = Path::new(m.value_of("FILENAME").unwrap());
    let cyclic = m.is_present("cyclic");

    // Get the name of the plan; either provided explicitly or
    // deduced from the file name
    let name = m.value_of("name").unwrap_or(match filename.file_stem() {
        Some(n) => {
            n.to_str().ok_or(Error::from_kind(ErrorKind::Utf8("invalid utf8 in filename".into())))?
        }
        None => {
            bail!("could not deduce plan name from filename '{}'",
                  filename.display())
        }
    });

    // Try to open the file and parse a plan from it
    let f = File::open(&filename).chain_err(|| ErrorKind::Io(format!("could not open file {}", filename.display())))?;
    let mut plan = Plan::from_text(name, &f).chain_err(|| "could not parse plan")?;

    if cyclic {
        plan.set_cyclic(true);
    }

    // Now add the plan to the plans directory
    files::add_plan(&plan).chain_err(|| "could not add plan")?;

    styleln!(style_set.normal, "Added plan {}", name);
    Ok(())
}

/// The `remove` subcommand logic
fn remove(m: &ArgMatches, style_set: &StyleSet) -> Result<()> {
    let name = m.value_of("PLAN").unwrap();

    files::remove_plan(name).chain_err(|| "could not remove plan")?;

    styleln!(style_set.normal, "Removed plan {}", name);
    Ok(())
}

/// The `export` subcommand logic.
fn export(m: &ArgMatches, style_set: &StyleSet) -> Result<()> {
    let name = m.value_of("PLAN").unwrap();
    let plan = files::read_plan(name).chain_err(|| "could not read plan")?;

    // Construct default output filename if we don't have one provided
    let output = match m.value_of("output") {
        Some(o) => o.to_owned(),
        None => plan.name().to_owned() + ".plan",
    };

    // Open the output file for writing, with an error if it already exists
    let path = Path::new(&output);
    if path.exists() {
        bail!("output file '{}' already exists; will not overwrite",
              output);
    }
    let file = File::create(path).chain_err(|| ErrorKind::Io("could not open output file".into()))?;

    // Now write the plan to the file
    plan.to_text(file).chain_err(|| "could not write to output file")?;
    styleln!(style_set.normal,
             "Wrote plan '{}' to '{}'",
             plan.name(),
             output);
    Ok(())
}

/// The `list` subcommand logic
fn list(style_set: &StyleSet) -> Result<()> {
    let plans = match files::plans() {
        Ok(p) => p,
        Err(Error(ErrorKind::NoConfigDirectory, _)) => {
            styleln!(style_set.normal,
                     "Could not find plans directory; this probably means you haven't run the \
                      program yet. To add plans, use `reading add` or run `reading help add` for \
                      help.");
            return Ok(());
        }
        Err(e) => return Err(e),
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

    // If there are no plans, say so
    if plan_list.is_empty() {
        styleln!(style_set.normal,
                 "No plans are installed; you can add some by running `reading add` (use \
                  `reading help add` for more information)");
        return Ok(());
    }
    // Now print out all the data
    for (name, current, len) in plan_list {
        // Check for end of plan (current > len indicates this)
        if current > len {
            println!("{} {}",
                     style!(style_set.title, "{}", name),
                     style!(style_set.normal, "(end of plan)"));
        } else {
            println!("{} {}",
                     style!(style_set.title, "{}", name),
                     style!(style_set.normal, "(entry {} of {})", current, len));
            println!("{} {}",
                     style!(style_set.title, "{}", name),
                     style!(style_set.normal, "(entry {} of {})", current, len));
        }
    }

    // Output any failures
    match failures {
        0 => {}
        1 => styleln!(style_set.error, "{}", "1 plan could not be read"),
        n @ _ => styleln!(style_set.error, "{} plans could not be read", n),

    }

    Ok(())
}

/// The `view` subcommand logic
fn view(m: &ArgMatches, style_set: &StyleSet) -> Result<()> {
    let name = m.value_of("PLAN").unwrap();
    // We can unwrap this because we set a default value
    let count =
        m.value_of("count").unwrap().parse().chain_err(|| "invalid numeric argument to `--count`")?;

    let plan = files::read_plan(name).chain_err(|| "could not read plan")?;

    // If we're at the end of the plan, indicate this
    if plan.is_ended() {
        styleln!(style_set.normal,
                 "Plan has ended (use `reading previous` to revert to an earlier entry)");
        return Ok(());
    }
    // Print out the given number of entries, starting at the current one
    for (n, entry) in plan.entries().skip(plan.current_entry_number() - 1).take(count).enumerate() {
        let label = match n {
            0 => "Current entry: ".to_owned(),
            1 => "Next entry: ".to_owned(),
            _ => format!("{} entries from now: ", n),
        };

        println!("{} {}",
                 style!(style_set.normal, "{:20}", label),
                 style!(style_set.title, "{}", entry.title()));
        if !entry.description().is_empty() {
            styleln!(style_set.description, "{:20} {}", "", entry.description());
        }
    }

    Ok(())
}

/// The `next` subcommand logic.
/// The `next` argument specifies whether the next operation is actually desired;
/// set this to false to get the `previous` subcommand logic, since it's
/// almost identical.
fn next(m: &ArgMatches, style_set: &StyleSet, next: bool) -> Result<()> {
    let name = m.value_of("PLAN").unwrap();
    let count =
        m.value_of("count").unwrap().parse().chain_err(|| "invalid numeric argument to `--count`")?;

    let mut plan = files::read_plan(name).chain_err(|| "could not read plan")?;

    // Go to next entry
    let old_entry = if plan.is_ended() {
        "end".to_owned()
    } else {
        plan.current_entry_number().to_string()
    };
    if next {
        plan.next(count);
    } else {
        plan.previous(count);
    }
    let new_entry = if plan.is_ended() {
        "end".to_owned()
    } else {
        plan.current_entry_number().to_string()
    };

    // Resave the plan after making this change
    files::overwrite_plan(&plan).chain_err(|| "could not overwrite plan")?;
    styleln!(style_set.normal,
             "Changed current entry of '{}': {} -> {}",
             plan.name(),
             old_entry,
             new_entry);

    Ok(())
}
