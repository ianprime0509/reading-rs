# reading
`reading` is a simple reading plan manager, but it can also manage any plans containing a series of entries (e.g. a project outline).
Right now, the basic functionality is present: you can add plans from plain text files, view their entries, and change the current entry of installed plans.
The location of installed plans is currently sub-optimal (will choose `~/.reading` as the storage directory), and should be changed to use the XDG specification, at least on Linux (not sure about other platforms).

Don't try to use this as a library right now, because I won't increment the version number until I think things are fine.

Run `reading help` for more information.
