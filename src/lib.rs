pub mod job;
pub mod job_builder;
pub mod job_post_processing;
mod job_status;
pub mod memory_size;
pub mod slurm_manager;

#[cfg(test)]
mod tests {
    use crate::job::SlurmJob;
    use crate::job_builder::SlurmJobBuilder;
    use crate::job_post_processing::SlurmJobPostProcessing;
    use crate::memory_size::Memory::MegaByte;
    use crate::slurm_manager::SlurmManager;

    fn get_post_processing() -> SlurmJobPostProcessing {
        SlurmJobPostProcessing::new(&[], |_| true)
    }

    fn generate_job() -> SlurmJob {
        SlurmJobBuilder::new("sleep 5".to_string())
            .set_working_directory("/home/user/".to_string())
            .set_cpus(1)
            .set_output_file("out.log".to_string())
            .set_error_file("error.log".to_string())
            .set_max_run_time("0-00:05:00".to_string())
            .set_memory(MegaByte(100))
            .set_on_finished(get_post_processing())
            .build()
    }

    #[test]
    #[ignore = "requires a live SLURM cluster (run with --include-ignored)"]
    fn test_slurm_manager() {
        let mut manager: SlurmManager = SlurmManager::new(3);
        for _ in 0..5 {
            manager.add_job(&generate_job());
        }
        manager.manage_jobs(None);
    }
}
