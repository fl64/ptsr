## ADDED Requirements

### Requirement: Benchmark overrun records

The offline callback benchmark SHALL automatically retain each measured block whose wall or thread CPU duration exceeds 50% of the block budget.
Each record SHALL include the phase, switch number, block number, monotonic offset and paired wall, CPU and estimated-gap durations.
Equality at 50% SHALL NOT count as an overrun.

#### Scenario: Benchmark retains every overrun identity
- **WHEN** a first or later block exceeds the target by wall time, CPU time or both
- **THEN** one row retains its original paired times and phase identity
- **AND** CPU-only overruns remain visible, while equality produces no row

### Requirement: Benchmark boundary observations

Each overrun record SHALL include available before/after CPU IDs and changes in thread context-switch and page-fault counters.
The report SHALL label these as observations across the larger measurement envelope, not exact callback events or proof of continuous CPU residence.
Unavailable or regressing observations SHALL carry an explicit reason instead of an invented value.

#### Scenario: Benchmark reports unavailable boundary data
- **WHEN** a CPU or resource read fails, or a resource counter moves backwards
- **THEN** affected fields explicitly report unavailable data and its reason
- **AND** the valid timing pair and other observations remain intact

### Requirement: Benchmark frequency observation

For each overrun, the benchmark SHALL attempt a frequency read for its after-CPU outside callback timing.
The row SHALL include the source and monotonic start/end of that read.
The report SHALL NOT claim that this later policy observation proves the frequency inside the block.

#### Scenario: Benchmark labels later frequency samples
- **WHEN** an overrun has an available after-CPU
- **THEN** its row reports the later frequency observation with source and read interval, or an explicit unavailable reason
- **AND** it does not infer the cause of the overrun from that observation

### Requirement: Benchmark local CSV output

The benchmark SHALL save one uniquely named local CSV under `target/bench-diagnostics/` without replacing an existing file.
It SHALL write a header, Unix run-start time, observation limitations and all buffered overrun rows before applying the existing p99 exit gates.
Output failure SHALL fail the command with a clear error.
Raw CSV files, detailed machine logs and identifying local paths SHALL NOT enter repository artifacts.
Anonymized verification summaries MAY remain in documents and commit history.

#### Scenario: Benchmark saves records before exit checks
- **WHEN** measurement completes with zero or more overruns, including a failing p99 check
- **THEN** the CSV remains available with its header and every recorded row before the exit decision
- **AND** an existing CSV remains unchanged

#### Scenario: Benchmark refuses an output failure
- **WHEN** the CSV cannot be created, written or flushed
- **THEN** the command fails and identifies the diagnostic output failure

### Requirement: Benchmark observation isolation

New observations SHALL wrap the existing measurement without changing sound processing, workload, first-block inclusion or p99 checks.
The benchmark SHALL report instrumented empty-call and full-envelope overhead separately from the existing overhead report.
It SHALL NOT subtract overhead, require privileges, access sound devices or change scheduling settings.

#### Scenario: Benchmark observations preserve sound and gates
- **WHEN** the same real callback runs directly and through the observation wrapper
- **THEN** its output samples match and the original timing function executes exactly once
- **AND** the existing p99 exit checks and raw timing values stay unchanged

#### Scenario: Benchmark reports observation overhead
- **WHEN** the benchmark runs its empty-call measurements
- **THEN** separate output describes the instrumented empty-call times and complete observation-envelope overhead
- **AND** it warns that the observations can affect subsequent execution
