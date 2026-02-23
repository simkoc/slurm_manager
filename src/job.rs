use crate::job_post_processing::SlurmJobPostProcessing;
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
        }
    }

    pub(crate) fn get_status(&self) -> SlurmJobStatus {
        self.status.clone()
    }

    #[allow(unused)]
    pub(crate) fn get_id(&self) -> &String {
        &self.id
    }

    pub(crate) fn run_post_processing(&self) -> SlurmJobStatus {
        if self.on_finished.check() {
            SlurmJobStatus::FINISHED
        } else {
            SlurmJobStatus::CRASHED
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

    pub(crate) fn generate_slurm_script(&self) -> String {
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
        ret += "\n\n";
        ret += "echo START: `date +%Y-%m-%dT%H:%M:%S%z`\n";
        ret += self.generate_slurm_commands().as_str();
        ret += "\necho END: `date +%Y-%m-%dT%H:%M:%S%z`\n";
        ret
    }
}
