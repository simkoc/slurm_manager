use std::collections::HashMap;

#[derive(Clone)]
pub struct SlurmJobPostProcessing {
    param: HashMap<String, String>,
    check: fn(&HashMap<String, String>) -> bool,
}

impl SlurmJobPostProcessing {
    pub fn new(
        param: &[(String, String)],
        check: fn(&HashMap<String, String>) -> bool,
    ) -> SlurmJobPostProcessing {
        let param = HashMap::<String, String>::from_iter(param.iter().cloned());
        SlurmJobPostProcessing { param, check }
    }

    pub(crate) fn check(&self) -> bool {
        (self.check)(&self.param)
    }

    pub fn do_nothing() -> SlurmJobPostProcessing {
        SlurmJobPostProcessing {
            param: HashMap::new(),
            check: |_| true,
        }
    }
}
