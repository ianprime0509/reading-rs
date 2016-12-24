# reading
`reading` is a simple reading plan manager, but it can also manage any plans containing a series of entries (e.g. a project outline).

You can add plans from plain text using the `reading add` command.
The format of the plan is as below:
```
Entry
    Description
Entry
Entry
```
The above represents a plan with three entries; the first of these has a description, providing more details.

By default, a plan is *acyclic*; you can change the current entry using the `reading next` or `reading previous` commands, and an acyclic plan will reach its end if you try to advance past the last entry (the "end of plan" state).
A plan can also be designated as *cyclic*, which means that it will run in a loop: for example, if a cyclic plan has three entries and is on its second entry, running `reading next {plan} -c 2` will result in the plan being "advanced" to the first entry.

For a list of the various available subcommands, run `reading help`.
You can also run `reading help {subcommand}` for information on a given subcommand.

## As a library
The core functionality is exposed as a crate, so that it can be reused.
Documentation is available within each module.
