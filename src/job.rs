use crate::job_post_processing::{PostProcessingOutcome, SlurmJobPostProcessing};
use crate::job_status::SlurmJobStatus;
use crate::memory_size::Memory;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Clone)]
pub struct SlurmJob {
    pub(crate) id: String,
    pub(crate) number: Option<i32>,
    pub(crate) command: String,
    pub(crate) working_directory: Option<String>,
    #[allow(unused)]
    pub(crate) env: HashMap<String, String>,
    #[allow(unused)]
    pub(crate) description: String,
    pub(crate) status: SlurmJobStatus,
    pub(crate) max_run_time: Option<String>, // D-HH:MM:SS
    pub(crate) output_file: Option<String>,
    pub(crate) error_file: Option<String>,
    pub(crate) on_finished: SlurmJobPostProcessing,
    pub(crate) memory: Memory,
    pub(crate) cpus: usize,
    pub(crate) pre_kill_signal: Option<(String, u32)>,
}

impl Display for SlurmJob {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}", self.id).as_str())
    }
}

impl SlurmJob {
    #[allow(unused)]
    pub fn new(
        command: String,
        description: String,
        on_finished: SlurmJobPostProcessing,
    ) -> SlurmJob {
        SlurmJob {
            id: Uuid::new_v4().to_string(),
            number: None,
            command,
            working_directory: None,
            env: HashMap::new(),
            description,
            status: SlurmJobStatus::CREATED,
            max_run_time: None,
            output_file: None,
            error_file: None,
            on_finished,
            memory: Memory::MegaByte(100),
            cpus: 1,
            pre_kill_signal: None,
        }
    }

    pub(crate) fn get_status(&self) -> SlurmJobStatus {
        self.status.clone()
    }

    #[allow(unused)]
    pub(crate) fn get_id(&self) -> &String {
        &self.id
    }

    // returns the resulting status plus whether the job signaled a fatal,
    // systemic error that should stop the rest of the queue.
    pub(crate) fn run_post_processing(&self) -> (SlurmJobStatus, bool) {
        match self.on_finished.check() {
            PostProcessingOutcome::Success => (SlurmJobStatus::FINISHED, false),
            PostProcessingOutcome::Failure => (SlurmJobStatus::CRASHED, false),
            PostProcessingOutcome::Fatal => (SlurmJobStatus::CRASHED, true),
        }
    }

    pub(crate) fn set_number(&mut self, number: i32) -> () {
        match self.number {
            Some(_) => panic!("must not overwrite existing job number"),
            None => self.number = Some(number),
        }
    }

    pub(crate) fn set_status(&mut self, status: SlurmJobStatus) -> () {
        self.status = status;
    }

    #[allow(unused)]
    pub(crate) fn get_number(&self) -> i32 {
        self.number.expect("no number set for the job")
    }

    /// The raw outcome of this job's post-processing callback, for callers that want to handle
    /// `Success`/`Failure`/`Fatal` themselves rather than through the `SlurmJobStatus` mapping
    /// `run_post_processing` applies (e.g. a non-SLURM backend with its own status model).
    pub fn post_processing_outcome(&self) -> PostProcessingOutcome {
        self.on_finished.check()
    }

    pub fn get_output_file(&self) -> Option<&str> {
        self.output_file.as_deref()
    }

    pub fn get_error_file(&self) -> Option<&str> {
        self.error_file.as_deref()
    }

    pub fn get_cpus(&self) -> usize {
        self.cpus
    }

    pub fn get_memory(&self) -> Memory {
        self.memory.clone()
    }

    /// `max_run_time` (`D-HH:MM:SS`) converted to a total second count, for callers that need to
    /// enforce the same limit outside of SLURM's own `--time` accounting (e.g. a plain-bash
    /// backend). `None` if no `max_run_time` was set.
    pub fn max_run_time_seconds(&self) -> Option<u64> {
        let max_run_time = self.max_run_time.as_ref()?;
        let parts: Vec<&str> = max_run_time.splitn(2, '-').collect();
        let days: u64 = parts[0]
            .parse()
            .expect("max_run_time days should be a valid number");
        let hms: Vec<&str> = parts[1].split(':').collect();
        let hours: u64 = hms[0]
            .parse()
            .expect("max_run_time hours should be a valid number");
        let minutes: u64 = hms[1]
            .parse()
            .expect("max_run_time minutes should be a valid number");
        let seconds: u64 = hms[2]
            .parse()
            .expect("max_run_time seconds should be a valid number");
        Some(days * 86400 + hours * 3600 + minutes * 60 + seconds)
    }

    pub(crate) fn generate_slurm_commands(&self) -> String {
        let mut ret = String::new();
        match self.working_directory {
            Some(ref working_directory) => {
                ret += format!("pushd {}\n", working_directory).as_str();
            }
            None => {}
        }
        ret += self.command.as_str();
        ret += "\n";
        if self.working_directory.is_some() {
            ret += "popd\n";
        }
        ret
    }

    pub fn generate_slurm_script(&self) -> String {
        let mut ret = String::from("#!/bin/bash\n");
        ret += format!("#SBATCH --job-name={}\n", self.id).as_str();
        match self.output_file {
            Some(ref output_file) => {
                ret += format!("#SBATCH --output={}\n", output_file).as_str();
            }
            None => {}
        }
        match self.error_file {
            Some(ref error_file) => {
                ret += format!("#SBATCH --error={}\n", error_file).as_str();
            }
            None => {}
        }
        ret += format!("#SBATCH --cpus-per-task={}\n", self.cpus).as_str();
        ret += match self.memory {
            Memory::MegaByte(memory) => format!("#SBATCH --mem={}M\n", memory),
            Memory::GigaByte(memory) => format!("#SBATCH --mem={}G\n", memory),
        }
        .as_str();
        match &self.max_run_time {
            Some(max_run_time) => ret += format!("#SBATCH --time={}\n", max_run_time).as_str(),
            None => {}
        }
        match &self.pre_kill_signal {
            Some((signal, seconds_before)) => {
                ret += format!("#SBATCH --signal=B:{}@{}\n", signal, seconds_before).as_str()
            }
            None => {}
        }
        ret += "\n\n";
        ret += "echo START: `date +%Y-%m-%dT%H:%M:%S%z`\n";
        ret += self.generate_slurm_commands().as_str();
        ret += "\necho END: `date +%Y-%m-%dT%H:%M:%S%z`\n";
        ret
    }
}

#[cfg(test)]
mod tests {
    use crate::job_builder::SlurmJobBuilder;
    use crate::job_post_processing::{PostProcessingOutcome, SlurmJobPostProcessing};
    use crate::memory_size::Memory;

    #[test]
    fn no_signal_flag_by_default() {
        let job = SlurmJobBuilder::new("sleep 1".to_string()).build();
        assert!(!job.generate_slurm_script().contains("--signal"));
    }

    #[test]
    fn pre_kill_signal_emits_sbatch_flag() {
        let job = SlurmJobBuilder::new("sleep 1".to_string())
            .set_pre_kill_signal("USR1", 30)
            .build();
        assert!(
            job.generate_slurm_script()
                .contains("#SBATCH --signal=B:USR1@30\n")
        );
    }

    #[test]
    fn post_processing_outcome_returns_success() {
        let job = SlurmJobBuilder::new("sleep 1".to_string())
            .set_on_finished(SlurmJobPostProcessing::new(&[], |_| {
                PostProcessingOutcome::Success
            }))
            .build();
        assert_eq!(
            job.post_processing_outcome(),
            PostProcessingOutcome::Success
        );
    }

    #[test]
    fn post_processing_outcome_returns_fatal() {
        let job = SlurmJobBuilder::new("sleep 1".to_string())
            .set_on_finished(SlurmJobPostProcessing::new(&[], |_| {
                PostProcessingOutcome::Fatal
            }))
            .build();
        assert_eq!(job.post_processing_outcome(), PostProcessingOutcome::Fatal);
    }

    #[test]
    fn get_output_file_defaults_to_dev_null() {
        let job = SlurmJobBuilder::new("sleep 1".to_string()).build();
        assert_eq!(job.get_output_file(), Some("/dev/null"));
    }

    #[test]
    fn get_output_file_returns_configured_path() {
        let job = SlurmJobBuilder::new("sleep 1".to_string())
            .set_output_file("out.log".to_string())
            .build();
        assert_eq!(job.get_output_file(), Some("out.log"));
    }

    #[test]
    fn get_error_file_defaults_to_dev_null() {
        let job = SlurmJobBuilder::new("sleep 1".to_string()).build();
        assert_eq!(job.get_error_file(), Some("/dev/null"));
    }

    #[test]
    fn get_error_file_returns_configured_path() {
        let job = SlurmJobBuilder::new("sleep 1".to_string())
            .set_error_file("err.log".to_string())
            .build();
        assert_eq!(job.get_error_file(), Some("err.log"));
    }

    #[test]
    fn get_cpus_defaults_to_one() {
        let job = SlurmJobBuilder::new("sleep 1".to_string()).build();
        assert_eq!(job.get_cpus(), 1);
    }

    #[test]
    fn get_cpus_returns_configured_value() {
        let job = SlurmJobBuilder::new("sleep 1".to_string())
            .set_cpus(4)
            .build();
        assert_eq!(job.get_cpus(), 4);
    }

    #[test]
    fn get_memory_defaults_to_100_megabytes() {
        let job = SlurmJobBuilder::new("sleep 1".to_string()).build();
        assert!(matches!(job.get_memory(), Memory::MegaByte(100)));
    }

    #[test]
    fn get_memory_returns_configured_value() {
        let job = SlurmJobBuilder::new("sleep 1".to_string())
            .set_memory(Memory::GigaByte(8))
            .build();
        assert!(matches!(job.get_memory(), Memory::GigaByte(8)));
    }

    #[test]
    fn max_run_time_seconds_is_none_when_unset() {
        let job = SlurmJobBuilder::new("sleep 1".to_string()).build();
        assert_eq!(job.max_run_time_seconds(), None);
    }

    #[test]
    fn max_run_time_seconds_converts_days_hours_minutes_seconds() {
        let job = SlurmJobBuilder::new("sleep 1".to_string())
            .set_max_run_time("1-02:03:04".to_string())
            .build();
        assert_eq!(
            job.max_run_time_seconds(),
            Some(86400 + 2 * 3600 + 3 * 60 + 4)
        );
    }

    #[test]
    fn max_run_time_seconds_handles_zero_days() {
        let job = SlurmJobBuilder::new("sleep 1".to_string())
            .set_max_run_time("0-00:05:00".to_string())
            .build();
        assert_eq!(job.max_run_time_seconds(), Some(300));
    }
}
