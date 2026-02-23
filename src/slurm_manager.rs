use crate::job::SlurmJob;
use crate::job_status::SlurmJobStatus;
use crate::job_status::SlurmJobStatus::{PENDING, SUBMITTED};
use chrono::{Local, TimeDelta};
use log::{error, info, warn};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum SlurmInteractionError {
    BadSbatchResponse(#[allow(unused)] String),
    SlurmUnresponsive(#[allow(unused)] String),
}

pub struct SlurmManager {
    open_jobs: Vec<SlurmJob>,
    scheduled_jobs: Vec<SlurmJob>,
    finished_jobs: Vec<SlurmJob>,
    max_queue: i32,
}

impl SlurmManager {
    pub fn new(max_queue: i32) -> SlurmManager {
        SlurmManager {
            open_jobs: Vec::new(),
            scheduled_jobs: Vec::new(),
            finished_jobs: Vec::new(),
            max_queue,
        }
    }

    pub fn add_job(&mut self, job: &SlurmJob) {
        let mut cloned = job.clone();
        cloned.set_status(PENDING);
        self.open_jobs.push(cloned);
    }

    #[allow(unused)]
    pub fn add_jobs(&mut self, jobs: Vec<SlurmJob>) {
        jobs.iter().for_each(|job| self.add_job(job))
    }

    pub fn successful_jobs(&self) -> i32 {
        self.finished_jobs
            .iter()
            .filter(|job| job.get_status() == SlurmJobStatus::FINISHED)
            .count() as i32
    }

    fn parse_squeue_row(row: &str) -> (i32, String, String, String, String, String, i32, String) {
        let row_split: Vec<&str> = row.split(" ").collect();
        if row_split.len() != 8 {
            panic!("unexpected row format: {}", row);
        }
        (
            //todo: we should also support arrays but do not do so yet
            row_split[0]
                .parse()
                .expect(format!("we need an integer at the first element: {}", row).as_str()),
            String::from(row_split[1]),
            String::from(row_split[2]),
            String::from(row_split[3]),
            String::from(row_split[4]),
            String::from(row_split[5]),
            row_split[6]
                .parse()
                .expect(format!("we need an integer at the sixth element: {}", row).as_str()),
            String::from(row_split[7]),
        )
    }

    fn get_running_jobs(&self) -> Result<HashSet<i32>, SlurmInteractionError> {
        let mut running_jobs: HashSet<i32> = HashSet::new();
        match std::process::Command::new("squeue")
            .args(["--me", "--format", "%.i %.P %.j %.u %.t %.M %.D %R"])
            .output()
        {
            Ok(output) => {
                let out = String::from_utf8(output.stdout).expect("squeue should return string");
                let split: Vec<&str> = out.split("\n").collect();
                for row in &split[1..] {
                    if row.len() == 0 {
                        continue;
                    }
                    let (id, _, _, _, _, _, _, _) = Self::parse_squeue_row(row);
                    running_jobs.insert(id);
                }
                Result::Ok(running_jobs)
            }
            Err(bad) => Err(SlurmInteractionError::SlurmUnresponsive(bad.to_string())),
        }
    }

    fn check_on_jobs(&mut self) -> Result<i32, SlurmInteractionError> {
        let running_jobs = self.get_running_jobs()?;
        let mut finished_jobs = 0;
        let mut done = Vec::new();
        for (index, job) in self.scheduled_jobs.iter().enumerate() {
            if !running_jobs.contains(&job.get_number()) {
                done.push(index);
                finished_jobs += 1;
            }
        }
        done.sort_by(|a, b| a.cmp(b));
        done.reverse();
        for elem in done {
            let mut finished_job = self.scheduled_jobs.remove(elem);
            let status = finished_job.run_post_processing();
            finished_job.set_status(status);
            self.finished_jobs.push(finished_job);
        }
        Result::Ok(finished_jobs)
    }

    fn schedule_job(&self, job: &mut SlurmJob) -> Result<i32, SlurmInteractionError> {
        let tmp_dir = match std::env::var("TMP_DIR") {
            Ok(tmp_dir) => tmp_dir,
            _ => String::from("/tmp/"),
        };
        let slurm_script = tmp_dir + "script.slurm";
        let mut slurm_file = File::create(&slurm_script).expect("Couldn't create slurm script");
        slurm_file
            .write(job.generate_slurm_script().as_bytes())
            .expect("Couldn't write to slurm script");
        slurm_file.flush().expect("Couldn't flush slurm script");
        slurm_file.sync_all().expect("Couldn't sync slurm script");
        match std::process::Command::new("sbatch")
            .arg(slurm_script)
            .output()
        {
            Ok(output) => {
                let mut out =
                    String::from_utf8(output.stdout).expect("Couldn't convert output to string");
                out = out.trim().to_string();
                let out_split = out.split(" ").collect::<Vec<&str>>();
                match out_split.last().unwrap().parse::<i32>() {
                    Ok(job_id) => {
                        job.set_status(SUBMITTED);
                        Ok(job_id)
                    }
                    Err(_) => Err(SlurmInteractionError::BadSbatchResponse(String::from(out))),
                }
            }
            Err(bad_status) => Err(SlurmInteractionError::SlurmUnresponsive(
                bad_status.to_string(),
            )),
        }
    }

    fn fill_up_queue(&mut self) -> Result<i32, Vec<SlurmInteractionError>> {
        let mut errors = Vec::<SlurmInteractionError>::new();
        let queue_delta = self.max_queue - self.scheduled_jobs.len() as i32;
        let mut added_jobs = 0;
        for _ in 0..queue_delta {
            match self.open_jobs.pop() {
                Some(mut job) => match self.schedule_job(&mut job) {
                    Ok(job_id) => {
                        job.set_number(job_id);
                        self.scheduled_jobs.push(job);
                        added_jobs += 1;
                    }
                    Err(e) => {
                        error!("encountered issue {:?}", e);
                        errors.push(e);
                    }
                },
                None => return Ok(added_jobs),
            }
        }
        if errors.len() == 0 {
            Ok(added_jobs)
        } else {
            Err(errors)
        }
    }

    // start scheduling jobs, return true if all jobs are done
    pub fn manage_jobs(&mut self, for_sec: Option<i64>) -> bool {
        let max_time_delta = 365 * 24 * 60; // one year worth of seconds
        let end_time = Local::now() + TimeDelta::seconds(for_sec.unwrap_or_else(|| max_time_delta));
        loop {
            // run loop until either the time is up
            if Local::now() >= end_time
                || (self.open_jobs.is_empty() && self.scheduled_jobs.is_empty())
            {
                break;
            }
            match self.check_on_jobs() {
                Result::Ok(finished_jobs) => {
                    info!("jobs finished since last check {}", finished_jobs);
                }
                Result::Err(why) => {
                    warn!("Error while checking on jobs: {:?}", why);
                }
            }
            match self.fill_up_queue() {
                Result::Ok(added_jobs) => {
                    if added_jobs > 0 {
                        info!("we scheduled {} new jobs", added_jobs);
                    }
                }
                Result::Err(why) => {
                    error!("while scheduling jobs we encountered {} errors", why.len());
                }
            }
            let time_remaining = end_time - Local::now();
            info!(
                "there are {} jobs remaining to be completed within the next {} seconds",
                self.open_jobs.len() + self.scheduled_jobs.len(),
                time_remaining.as_seconds_f32()
            );
            thread::sleep(Duration::from_secs(5)); // wait for 5 seconds and then update jobs
        }
        self.open_jobs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::job_builder::SlurmJobBuilder;
    use crate::job_post_processing::SlurmJobPostProcessing;
    use log4rs::config::Deserializers;
    use serial_test::serial;
    //use crate::logging::Logger;
    use super::*;

    fn init_logger() {
        //todo: do we need to init anything here?
    }

    fn always_success() -> SlurmJobPostProcessing {
        SlurmJobPostProcessing::new(&[], |_| true)
    }

    fn sleep_job(wdir: Option<String>) -> SlurmJob {
        let job = SlurmJobBuilder::new(String::from("sleep 5"))
            .set_description(String::from("sleeps for 5 seconds"));
        match wdir {
            Some(dir) => job.set_working_directory(String::from(dir)).build(),
            None => job.build(),
        }
    }

    #[test]
    fn generate_job_command() {
        let job = sleep_job(None);
        assert_eq!(job.generate_slurm_commands(), "sleep 5\n");
    }

    #[test]
    fn generate_job_command_wdir() {
        let job = sleep_job(Some("/tmp/".parse().unwrap()));
        assert_eq!(
            job.generate_slurm_commands(),
            r"pushd /tmp/
sleep 5
popd
"
        );
    }

    #[test]
    fn generate_full_script() {
        let job = sleep_job(None);
        let mut expected: String = String::from("#!/bin/bash\n");
        expected += format!("#SBATCH --job-name={}\n\n\nsleep 5\n", job.get_id()).as_str();
        assert_eq!(job.generate_slurm_script(), expected);
    }

    #[test]
    #[serial]
    fn create_and_run_jobs() {
        let job = sleep_job(None);
        init_logger();
        let mut manager = SlurmManager::new(2);
        manager.add_job(&job);
        let pre_start = manager.check_on_jobs().expect("Should have checked no job");
        let scheduled = manager.fill_up_queue().expect("Couldn't fill up queue");
        let running = manager.get_running_jobs().expect("get running jobs").len();
        let done = manager.manage_jobs(Some(10));
        assert_eq!(pre_start, 0);
        assert_eq!(scheduled, 1);
        assert_eq!(running, 1);
        assert!(done);
    }

    #[test]
    #[serial]
    fn create_and_run_multiple_jobs() {
        let job_one = sleep_job(None);
        let job_two = sleep_job(None);
        init_logger();
        let mut manager = SlurmManager::new(2);
        manager.add_jobs(Vec::from([job_one, job_two]));
        let pre_start = manager.check_on_jobs().expect("Should have checked no job");
        let scheduled = manager.fill_up_queue().expect("Couldn't fill up queue");
        let running = manager.get_running_jobs().expect("get running jobs").len();
        let done = manager.manage_jobs(Some(10));
        assert_eq!(pre_start, 0);
        assert_eq!(scheduled, 2);
        assert_eq!(running, 2);
        assert!(done);
    }
}
