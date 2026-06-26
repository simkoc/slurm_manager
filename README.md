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
    .build();
```

### Submit and manage jobs

```rust
use slurm_manager::slurm_manager::SlurmManager;

let mut manager = SlurmManager::new(3); // at most 3 jobs queued in SLURM at once

for _ in 0..10 {
    manager.add_job(&job);
}

// blocks until all jobs finish; pass Some(seconds) to set a timeout
let all_done: bool = manager.manage_jobs(None);
```

`manage_jobs` polls `squeue` every 5 seconds, fills the queue up to `max_queue`, and runs the post-processing callback for each finished job. It returns `true` if every job completed before the timeout.

### Post-processing

`SlurmJobPostProcessing` runs a callback after each job disappears from `squeue`. Return `true` for success, `false` to mark the job as crashed. Use the parameter map to pass context (e.g. expected output paths to verify).

```rust
let post = SlurmJobPostProcessing::new(
    &[("output".to_string(), "/tmp/result.txt".to_string())],
    |params| std::path::Path::new(&params["output"]).exists(),
);
```

Jobs marked crashed are excluded from `manager.successful_jobs()`.

## Running tests

```bash
# offline unit tests (no SLURM required)
cargo test

# integration tests (require a live SLURM cluster)
cargo test -- --include-ignored
```

The `TMP_DIR` environment variable controls where temporary `.slurm` scripts are written (default: `/tmp/`).

## Local SLURM setup (Arch Linux)

```bash
pacman -S slurm-llnl
# edit /etc/slurm-llnl/slurm.conf to match your hardware
systemctl start munge slurmctld slurmd
sinfo  # should show a node in 'idle' state
```
