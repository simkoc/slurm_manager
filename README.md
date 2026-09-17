# slurm_manager

A Rust library for creating and managing SLURM batch jobs. It wraps `sbatch` and `squeue` to submit jobs, keep a bounded queue filled, and run post-processing callbacks when jobs finish.

## Installation

```toml
slurm_manager = { git = "https://github.com/simkoc/slurm_manager" }
```

## Usage

### Build a job

```rust
use slurm_manager::job::SlurmJob;
use slurm_manager::job_builder::SlurmJobBuilder;
use slurm_manager::job_post_processing::SlurmJobPostProcessing;
use slurm_manager::memory_size::Memory::MegaByte;

let post = SlurmJobPostProcessing::new(&[], |_| true);

let job: SlurmJob = SlurmJobBuilder::new("sleep 5".to_string())
    .set_working_directory("/home/user/".to_string())
    .set_cpus(1)
    .set_output_file("out.log".to_string())
    .set_error_file("error.log".to_string())
    .set_max_run_time("0-00:05:00".to_string()) // D-HH:MM:SS
    .set_memory(MegaByte(100))
    .set_on_finished(post)
    // optional: send a catchable signal to the whole batch job 30s before
    // it's killed for exceeding --time, giving it a chance to flush output.
    .set_pre_kill_signal("USR1", 30)
    .build();
```

### Submit and manage jobs

```rust
use slurm_manager::slurm_manager::SlurmManager;

// at most 3 jobs queued in SLURM at once; `script_dir` must already exist and is where each
// job's `.slurm` submission script is written (as a uniquely-named file) before `sbatch` reads
// it - point it at a directory only this process (or a trusted set of processes) can write to.
let mut manager = SlurmManager::new(3, "/home/user/slurm_scripts");

for _ in 0..10 {
    manager.add_job(&job);
}

// blocks until all jobs finish; pass Some(seconds) to set a timeout.
// abort_if is checked each loop iteration; return true to stop submitting
// new jobs, scancel everything in flight, and mark remaining jobs ABORTED.
use slurm_manager::slurm_manager::ManageJobsOutcome;
let outcome: ManageJobsOutcome = manager.manage_jobs(None, &|| false);
```

`manage_jobs` polls `squeue` every 5 seconds, fills the queue up to `max_queue`, and runs the post-processing callback for each finished job. It returns a `ManageJobsOutcome`: `AllFinished`, `TimedOut`, or `Aborted` (when `abort_if` or a job's post-processing signaled).

### Post-processing

`SlurmJobPostProcessing` runs a callback after each job disappears from `squeue`. It returns a `PostProcessingOutcome`: `Success`, `Failure` (marks the job crashed), or `Fatal` (marks it crashed *and* tells `manage_jobs` to stop submitting further jobs, `scancel` everything in flight, and return `Aborted` — use this when a job detects a systemic problem, e.g. a shared setup step failing, that would doom the rest of the queue the same way). Use the parameter map to pass context (e.g. expected output paths to verify).

```rust
let post = SlurmJobPostProcessing::new(
    &[("output".to_string(), "/tmp/result.txt".to_string())],
    |params| {
        if std::path::Path::new(&params["output"]).exists() {
            PostProcessingOutcome::Success
        } else {
            PostProcessingOutcome::Failure
        }
    },
);
```

Jobs marked crashed or aborted are excluded from `manager.successful_jobs()`.

## Running tests

```bash
# offline unit tests (no SLURM required)
cargo test

# integration tests (require a live SLURM cluster)
cargo test -- --include-ignored
```

## Local SLURM setup (Arch Linux)

```bash
pacman -S slurm-llnl
# edit /etc/slurm-llnl/slurm.conf to match your hardware
systemctl start munge slurmctld slurmd
sinfo  # should show a node in 'idle' state
```
