/// Represents a single entry in a reading plan, containing
/// a title and an (optional) description.
#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    title: String,
    description: String,
}

/// Represents a single reading plan. Each plan has a name and a list of
/// `Entry`s, and keeps track of the current entry in the plan.
#[derive(Serialize, Deserialize, Debug)]
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
