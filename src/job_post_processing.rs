use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostProcessingOutcome {
    Success,
    Failure,
    // the job detected a systemic problem (e.g. shared setup step failed)
    // that will affect every other queued job the same way; signals the
    // manager to stop submitting further jobs and cancel those in flight.
    Fatal,
}

#[derive(Clone)]
pub struct SlurmJobPostProcessing {
    param: HashMap<String, String>,
    check: fn(&HashMap<String, String>) -> PostProcessingOutcome,
}

impl SlurmJobPostProcessing {
    pub fn new(
        param: &[(String, String)],
        check: fn(&HashMap<String, String>) -> PostProcessingOutcome,
    ) -> SlurmJobPostProcessing {
        let param = HashMap::<String, String>::from_iter(param.iter().cloned());
        SlurmJobPostProcessing { param, check }
    }

    pub(crate) fn check(&self) -> PostProcessingOutcome {
        (self.check)(&self.param)
    }

    pub fn do_nothing() -> SlurmJobPostProcessing {
        SlurmJobPostProcessing {
            param: HashMap::new(),
            check: |_| PostProcessingOutcome::Success,
        }
    }
}
