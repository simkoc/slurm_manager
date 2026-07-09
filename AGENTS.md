# AGENTS.md

Guidelines for AI agents (and human contributors) working in this repository.

## Project Overview

`slurm_manager` is a Rust library for creating and managing SLURM batch jobs. The core types are:

- `SlurmJob` — a description of a single job (command, resources, post-processing).
- `SlurmJobBuilder` — builder pattern for constructing `SlurmJob` instances.
- `SlurmManager` — queues and tracks jobs against a live SLURM cluster via `sbatch` / `squeue`.
- `SlurmJobPostProcessing` — a callback invoked after a job finishes to determine success or failure.

The library interacts with SLURM only through `std::process::Command` (`sbatch`, `squeue`). All cluster I/O is isolated to `slurm_manager.rs`.

## Code Philosophy

- **Never fail silently.** Every `Result` and `Option` must be handled explicitly. Prefer `?` or `expect("reason")`; use `panic!` with a descriptive message for unrecoverable states.
- **Fail early.** Validate inputs in constructors and builder setters, not deep in call chains. Reject invalid state before it can propagate.
- **One function, one thing.** If you need "and" to describe what a function does, split it.
- **Refactor functions exceeding 40 lines.** Length is a signal to extract a named helper.
- **Test every non-trivial function** with two success cases (happy path + one variant) and one failure/edge case. Tests requiring a live SLURM cluster must be marked `#[ignore]` and `#[serial]`; all other tests must run offline.

## Development Notes

- The `TMP_DIR` environment variable controls where temporary `.slurm` script files are written (default: `/tmp/`).
- `SlurmJobPostProcessing` callbacks determine whether a finished job counts as `FINISHED` or `CRASHED`.
- The `env` field on `SlurmJob` is stored but not emitted into the generated script. Do not rely on it until that gap is closed.
