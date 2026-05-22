/// An enum with a single case.
#[derive(
    Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, serde::Serialize, serde::Deserialize,
)]
pub enum SingleCase {
    /// The single case.
    One,
}
