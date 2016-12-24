/// Represents a single entry in a reading plan, containing
/// a title and description, the latter of which may be empty.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Entry {
    title: String,
    description: String,
}

/// Represents a single reading plan.
///
/// Each plan has a name and a list of `Entry`s, and keeps track of the
/// current entry in the plan. Currently, you can construct a `Plan` by
/// reading it from plain text input (using the `from_text`
/// method), constructing it directly from a `Vec<Entry>` (using the `from_entries`
/// method), or by deserializing it from JSON. Likewise, it is possible
/// to output the `Plan` to plain text using the `to_text` method
/// or by serializing it using `serde`.
///
/// Besides just being a list of entries, a `Plan` also contains information
/// determining the behavior of certain operations on it. Currently, the
/// only data of this type is the `cyclic` property; if a plan is cyclic,
/// it will "wrap around" when using the `next` or `previous` methods;
/// if a plan is not cyclic, it will stop at the beginning of the plan
/// or at a designated "end of plan" state.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Plan {
    name: String,
    /// Whether the plan is cyclic, i.e. whether it will wrap around
    /// when the user tries to advance it past a certain entry
    cyclic: bool,
    /// The current entry of the plan, as a 0-based index (i.e. the first
    /// entry of the plan is 0). This can be equal to `entries.len()` to
    /// represent "end of plan", for a plan which is not cyclic.
    current_entry: usize,
    entries: Vec<Entry>,
}
