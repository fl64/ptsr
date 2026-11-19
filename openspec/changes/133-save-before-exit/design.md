# Design

## Evidence

The UI loop can close before `save_on_exit` knows whether a write succeeded.
With a prior write failure, that function skips the write.
See `crates/app/src/main.rs` and `crates/app/src/saving.rs`.

In the audit at `6d166b4`, a write into a disposable project failed.
The app showed `Save failed` and a master value of +2 dB.
A raw MIDI Exit confirmation closed the app. The saved file still held +1 dB.

## Owner decision

On 2026-11-19 the owner approved the specification for safe exit after a save failure.
The owner must see the current session, the reason and the next action before a press.
The recovery question offers `Retry Save` and `Back`.

## Exit sequence

1. Use Ctrl+Q for a direct exit request. Use the existing Exit question on SMC-PAD.
2. After the exit request, finish or cancel A1 capture at its current boundary.
3. Wait for an existing manual save of that session if one is pending.
4. If the session is clean, close ptsr.
5. If pending work remains after an earlier refusal, cancel exit and show the recovery question.
6. Otherwise save the pending work once. Close ptsr after success.
7. If the required save fails, cancel the exit request and show the recovery question.

A pending manual save retains its original input source. The exit request joins that save.
A successful save that clears pending work lets the app close.
A failed save retains the completed take and its Undo step.

## Recovery

Show the project name or `Scratch`, the save failure reason and the effective controls.
Enter or the SMC-PAD question confirm pad selects `Retry Save`.
Esc or the SMC-PAD question cancel pad selects `Back`.
Use the existing question rules for other input and releases.

`Retry Save` uses the current manual save path and requests one attempt.
A successful retry shows `Session saved`. The owner can leave with a new Exit request.
A failed retry shows the current reason and keeps the recovery question.
`Back` returns to the same screen with the work in memory.

## Cases to test

| Situation | Required result | Status |
| --- | --- | --- |
| Save failed before Exit and storage is available again | Show recovery. Wait for an explicit retry. | Specified |
| The first write for Exit fails | Keep the session and its last saved file. Show the reason. | Specified |
| A manual save still waits for A1 when Exit starts | Join its attempt. Retain its source and result. | Specified |
| A Gate note is active at the exit boundary | Keep the complete accepted take and one Undo step if the save fails. | Specified |
| Read protection applies or the project folder is missing | Respect the save refusal and keep the work. | Specified |
| The owner repeats a retry while a write is pending | Join the attempt. Wait for the result. | Specified |
| The owner cancels recovery | Keep pending work and the reason for a later Exit. | Specified |

Keep the last save reason available after its notice expires.
Use the current save path for file replacement, read protection and missing folders.
The boundary applies to requests within ptsr. An operating system can close the process.

## Open questions

Should a successful `Retry Save` also close ptsr?
The proposed default keeps ptsr open and shows `Session saved`.
A separate Exit request can then close the saved session.
