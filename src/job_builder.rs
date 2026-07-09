//todo: add capability to add modules on startup

use crate::job::SlurmJob;
use crate::job_post_processing::SlurmJobPostProcessing;
use crate::job_status::SlurmJobStatus;
use crate::memory_size::Memory;
use std::collections::HashMap;
use uuid::Uuid;

pub struct SlurmJobBuilder {
    command: String,
    working_directory: Option<String>,
    env: HashMap<String, String>,
    description: String,
    max_run_time: Option<String>,
    output_file: Option<String>,
    error_file: Option<String>,
    on_finished: SlurmJobPostProcessing,
    memory: Memory,
    cpus: usize,
}

impl SlurmJobBuilder {
    pub fn new(command: String) -> SlurmJobBuilder {
        SlurmJobBuilder {
            command,
            working_directory: None,
            env: HashMap::new(),
            description: String::from(""),
            max_run_time: None,
            output_file: Some("/dev/null".to_string()),
            error_file: Some("/dev/null".to_string()),
            on_finished: SlurmJobPostProcessing::do_nothing(),
            memory: Memory::MegaByte(100),
            cpus: 1,
        }
    }

    pub fn set_memory(mut self, memory: Memory) -> SlurmJobBuilder {
        self.memory = memory;
        self
    }

    pub fn set_working_directory(mut self, dir: String) -> SlurmJobBuilder {
        self.working_directory = Some(dir);
        self
    }

    #[allow(unused)]
    pub fn add_env(mut self, key: String, value: String) -> SlurmJobBuilder {
        self.env.insert(key, value);
        self
    }

    #[allow(unused)]
    pub fn set_description(mut self, desc: String) -> SlurmJobBuilder {
        self.description = desc;
        self
    }

    // Validates the SLURM time format D-HH:MM:SS required by --time.
    fn check_max_runtime_pattern(pattern: &str) -> bool {
        let parts: Vec<&str> = pattern.splitn(2, '-').collect();
        if parts.len() != 2 {
            return false;
        }
        if parts[0].parse::<u32>().is_err() {
            return false;
        }
        let hms: Vec<&str> = parts[1].split(':').collect();
        if hms.len() != 3 {
            return false;
        }
        let hours: u32 = match hms[0].parse() {
            Ok(v) => v,
            Err(_) => return false,
        };
        let minutes: u32 = match hms[1].parse() {
            Ok(v) => v,
            Err(_) => return false,
        };
        let seconds: u32 = match hms[2].parse() {
            Ok(v) => v,
            Err(_) => return false,
        };
        hours < 24 && minutes < 60 && seconds < 60
    }

    pub fn set_max_run_time(mut self, max_run_time: String) -> SlurmJobBuilder {
        assert!(
            Self::check_max_runtime_pattern(&max_run_time),
            "invalid max_run_time format, expected D-HH:MM:SS, got: {}",
            max_run_time
        );
        self.max_run_time = Some(max_run_time);
        self
    }

    pub fn set_output_file(mut self, output_file: String) -> SlurmJobBuilder {
        self.output_file = Some(output_file);
        self
    }

    pub fn set_error_file(mut self, error_file: String) -> SlurmJobBuilder {
        self.error_file = Some(error_file);
        self
    }

    pub fn set_on_finished(mut self, finished: SlurmJobPostProcessing) -> SlurmJobBuilder {
        self.on_finished = finished;
        self
    }

    pub fn set_cpus(mut self, cpus: usize) -> SlurmJobBuilder {
        self.cpus = cpus;
        self
    }

    pub fn build(&self) -> SlurmJob {
        SlurmJob {
            id: Uuid::new_v4().to_string(),
            number: None,
            command: self.command.clone(),
            working_directory: self.working_directory.clone(),
            env: self.env.clone(),
            description: self.description.clone(),
            status: SlurmJobStatus::CREATED,
            max_run_time: self.max_run_time.clone(),
            output_file: self.output_file.clone(),
            error_file: self.error_file.clone(),
            on_finished: self.on_finished.clone(),
            memory: self.memory.clone(),
            cpus: self.cpus,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_runtime_pattern_valid_zero_days() {
        assert!(SlurmJobBuilder::check_max_runtime_pattern("0-00:00:00"));
    }

    #[test]
    fn max_runtime_pattern_valid_nonzero() {
        assert!(SlurmJobBuilder::check_max_runtime_pattern("3-12:30:59"));
    }

    #[test]
    fn max_runtime_pattern_invalid_format() {
        assert!(!SlurmJobBuilder::check_max_runtime_pattern("00:05:00"));
        assert!(!SlurmJobBuilder::check_max_runtime_pattern("not-a-time"));
        assert!(!SlurmJobBuilder::check_max_runtime_pattern("1-25:00:00"));
        assert!(!SlurmJobBuilder::check_max_runtime_pattern("1-00:60:00"));
    }

    #[test]
    #[should_panic(expected = "invalid max_run_time format")]
    fn set_max_run_time_panics_on_bad_input() {
        SlurmJobBuilder::new("sleep 1".to_string()).set_max_run_time("badformat".to_string());
    }
}
