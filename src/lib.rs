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
    use log::info;

    fn get_post_processing() -> SlurmJobPostProcessing {
        SlurmJobPostProcessing::new(&[], |param| {
            println!("I finished successfully",);
            true
        })
    }

    fn generate_job() -> SlurmJob {
        SlurmJobBuilder::new("sleep 5".to_string()) // job is to sleep for 10 seconds
            .set_working_directory("/home/user/".to_string()) // run in home directory of user
            .set_cpus(1) // run on a single cpu
            .set_output_file("out.log".to_string()) // name of the output log
            .set_error_file("error.log".to_string()) // name of the error log
            .set_max_run_time(String::from("0-00:05:00".to_string())) // run for a maximum of five minutes
            .set_memory(MegaByte(100)) // use at most 100 MB of RAM
            .set_on_finished(get_post_processing()) // generate the post processor script
            .build() // build the job
    }

    #[test]
    fn test_slurm_manager() {
        // create our manager
        let mut manager: SlurmManager = SlurmManager::new(3);
        // add all jobs
        for i in 0..5 {
            manager.add_job(&generate_job());
        }
        // manage jobs until all jobs are done
        manager.manage_jobs(None);
    }
}
