# Slurm Manager

This crate provides utility to dynamically create and manage slurm jobs to run on a slurm managed cluster.

## Installation

Add this crate as a dependency to your Cargo.toml

```toml
slurm_manager = { git = "https://github.com/simkoc/slurm_manager" }
```

## Usage

This crate consists of two main components that provide the functionality.
We first have the `SlurmManager` that manages job creation and monitoring.
Second we have the `SlumJob` itself that represents a single job to be run via Slurm.
The `SlurmJob` in turn is created via its corresponding builder `SlurmJobBuilder`.

### The SlurmManager

The primary tool to create and manage our slurm job queue is the `SlurmManager`.
We create a new manager via `SlurmManager::new(<max_queue_lentgh>)` with `<max_queue_length>`
denoting how many jobs are supposed to be scheduled with slurm on the cluster
*(NOTE: This does not limit the amount of jobs that can be queued in the manager itself)*.
Subsequently, we can add jobs to the manager using `add_job(<slurm_job>)` or `add_jobs(<slurm_jobs>)`.
After we have added our jobs (we may add jobs later on if we want) we can tell the manager to 
start managing the jobs on the cluster via `manage_jobs(<for_sec>)` with `<for_sec>` being 
an optional integer denoting the amount of seconds until the function returns unless the jobs are 
not done first. In case of `None` the function returns if all scheduled jobs have been finished.
The call returns a boolean indicating if after termination all jobs are done.

```rust
// create our manager
let mut manager : SlurmManager = SlurmManager::new(3);
// add all jobs
for i in 0 .. 5 {
    manager.add_job(&generate_job());
}
// manage jobs until all jobs are done
manager.manage_jobs(None);
```

### The SlurmJob

While the manager manages the core element describing the actual work to be scheduled is the `SlurmJob`.
To create a new `SlurmJob` we use its builder instantiated via `SlurmJobBuilder::new(<command>)`.
`<command>` specifies the commandline commands that define the slurm job.
We then use the builder to add required execution parameters of the slurm job:
- `set_work_directory(<path>)` : the working directory of the slurm job
- `set_cpus(<int>)` : the amount of cpus the slurm job may use
- `set_output_file(<path>)` : the output file for the stdout of the slurm job
- `set_error_file(<path>)` : the output file for the stderr of the slurm job
- `set_max_run_time(<time-string>)` : time string `d-hh:mm:ss` denoting the maximum runtime of the slurm job
- `set_memory(<MemorySize>)` : the maximum amount of memory the slurm job may use
- `set_on_finished(<SlurmJobPostProcessing>)` : post-processing routine to be run when the slurm job finished

After having set all the required parameter we call `build()` and get a job we can pass on to our manager instance.

```rust
fn generate_job() -> SlurmJob {
    SlurmJobBuilder::new("sleep 10".to_string()) // job is to sleep for 10 seconds
        .set_working_directory("/home/user/".to_string()) // run in home directory of user
        .set_cpus(1) // run on a single cpu
        .set_output_file("out.log".to_string()) // name of the output log
        .set_error_file("error.log".to_string()) // name of the error log
        .set_max_run_time(String::from("0-00:05:00".to_string())) // run for a maximum of five minutes
        .set_memory(MegaByte(100)) // use at most 100 MB of RAM
        .set_on_finished(get_post_processing()) // generate the post processor script 
        .build() // build the job
}
```

### SlurmJobPostProcessing

The `SlurmJobPostProcessing` is a facility to run a custom routine after the `SlurmJob` finished.
We create one by using `SlurmJobPostProcessing::new(<params>,<routine>)`.
`<params>` is a hashmap containing key value pairs that are passed to the routine upon execution.
`<routine>` is a lambda function taking a hashmap reference returning a boolean to determine whether
the success of the post-processing, adding an additional option to sanity check the result of the slurm job.

```rust
fn get_post_processing() -> SlurmJobPostProcessing{
    SlurmJobPostProcessing::new(
        &[],
        |param| {
            println!("I finished successfully",);
            true
        }
    )
}
```

## Local Testing

We have the example code given above as a unit test suite in `lib.rs`.
Thus, we can check out the functionality and play around with it locally.
However, we need to set up our local machine for slurm unless it is not already configured.
Below, we will go through the required configuration steps for arch linux.
Other distributions probably work similar but your mileage might vary.
If you use mac or windows there will be dragons.

### Installation

```bash
$> pacman -S slurm-lnll
```

### Configuration

Now we go into `/etc/slurm-lnll/slurm.conf` and adjust the default values as required.
Given that the configuration is depending on your hardware, there is no point in elaborating here.
Any slurm tutorial of choice provides guidance.

### Starting

We start our configured slurm manager via

```bash
$> systemctl start munge
$> systemctl start slurmctld
$> systemctrl start slurmd
```

and can check whether it is up and running via

```bash
$> sinfo
PARTITION    AVAIL TIMELIMIT   NODES STATE   NODELIST
local*       up    infinite    1     idle    localhost
```


### Running the Test

```bash
$> cargo test test_slurm_manager -- --show-output
...f00...
running 1 test
test tests::test_slurm_manager ... ok

successes:

---- tests::test_slurm_manager stdout ----
I finished successfully
I finished successfully
I finished successfully
I finished successfully
I finished successfully


successes:
    tests::test_slurm_manager

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 25.08s

```

and in a different shell, while the unit test is running, run `squeue` to see the currently scheduled and running jobs.
If we increase the amount of jobs that can be scheduled at the same time we will at some point reach a limit
where the `ST` value is an `S` indicating that the job is `S`cheduled but not `R`unning.

```
$> squeue
squeue 
JOBID PARTITION     NAME     USER ST       TIME  NODES NODELIST(REASON)
401     local 95f37e9c    simon  R       0:02      1 roxanne
402     local 115de610    simon  R       0:02      1 roxanne
403     local 20c0c8b2    simon  R       0:02      1 roxanne
```