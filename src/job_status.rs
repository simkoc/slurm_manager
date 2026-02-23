#[derive(Clone, PartialEq, Eq)]
pub(crate) enum SlurmJobStatus {
    CREATED,
    PENDING,
    SUBMITTED,
    FINISHED,
    CRASHED,
}
