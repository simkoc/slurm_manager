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

    fn check_max_runtime_pattern(_pattern: &String) -> bool {
        true
    }

    pub fn set_max_run_time(mut self, max_run_time: String) -> SlurmJobBuilder {
        assert!(Self::check_max_runtime_pattern(&max_run_time));
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
