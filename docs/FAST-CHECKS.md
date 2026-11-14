# Test time measurements

These measurements use one machine with 14 logical CPUs. Each run used a warm build and
cargo-nextest 0.9.152. Wall time includes test execution. Test times overlap because tests
run in parallel. The base is `master` at `d5f6673`.

| Step | Tests | Wall time |
| --- | ---: | ---: |
| Base debug profile | 1,644 | 247.22 s |
| Optimized dependency and test profiles | 1,644 | 72.45 s |
| Scenario index and test work, before groups | 1,648 | 84.05 s |
| Slow group with two concurrent tests | 1,648 | 111.80 s |
| Slow group with four concurrent tests | 1,648 | 74.72 s |
| Fast set, after all test changes | 1,367 | 8.27 s |
| Typical changed set, `input_tests::feedback` | 38 | 0.96 s |

The first run with optimized profiles failed two catalogue tests. A pad flash expired
between two renders because those tests used wall time. Both tests passed after the
catalogue used one fixed fixture time.

The scenario coverage test took 29.33 s on the base. It took 1.34 s with the new profiles
and 0.09 s after the name index replaced repeated source searches. The base `ui_hangs`
test waited two seconds in real time. It now keeps the UI sender idle while it processes
two seconds of audio without that wait.

The table gives the ten longest tests in each full run. App test names omit the common
`ptsr::ptsr$` prefix. The measured full run used four concurrent slow tests. It came
before the idle UI test fix. Engine tests and the fast set passed after that fix.

| Rank | Base test | Base | Final test | Final |
| ---: | --- | ---: | --- | ---: |
| 1 | `screens::tests::pattern_titles_retain_safety_and_actual_playback_slots` | 128.97 s | `screens::tests::pattern_titles_retain_safety_and_actual_playback_slots` | 30.50 s |
| 2 | `screens::tests::catalogue_bytes_are_repeatable_without_devices_or_project_writes` | 97.09 s | `screens::tests::recording_feedback_catalogue_has_phases_controls_blockers_and_safety` | 22.56 s |
| 3 | `screens::tests::recording_feedback_catalogue_has_phases_controls_blockers_and_safety` | 95.83 s | `screens::tests::catalogue_covers_screens_and_existing_modes` | 20.57 s |
| 4 | `input_tests::badges::pads_play_badge_keeps_voice_identity_beside_pad_feedback` | 94.71 s | `screens::tests::pattern_warning_recovery_and_safety_remain_visible_with_long_project_names` | 15.38 s |
| 5 | `engine::pattern_count_in::count_in_before_the_first_recorded_beat` | 66.76 s | `input_tests::badges::pads_play_badge_keeps_voice_identity_beside_pad_feedback` | 12.91 s |
| 6 | `screens::tests::catalogue_covers_screens_and_existing_modes` | 63.61 s | `engine::pattern_count_in::count_in_before_the_first_recorded_beat` | 12.81 s |
| 7 | `input_tests::patterns::complete_recording_every_grid_and_strength_changes_future_sound_without_rewriting_raw_rows` | 55.10 s | `input_tests::deferred_release_follows_its_actual_pad_across_modes` | 11.19 s |
| 8 | `input_tests::deferred_release_follows_its_actual_pad_across_modes` | 53.05 s | `input_tests::late_rejected_direct_midi_notification_stays_silent_on_chop` | 8.58 s |
| 9 | `screens::tests::pattern_warning_recovery_and_safety_remain_visible_with_long_project_names` | 50.89 s | `input_tests::patterns::complete_recording_every_grid_and_strength_changes_future_sound_without_rewriting_raw_rows` | 8.26 s |
| 10 | `input_tests::patterns::app_erasure_unpublished_onsets_use_each_retained_grid_and_strength` | 45.38 s | `input_tests::late_accepted_direct_midi_notification_does_not_restart_chop` | 8.04 s |

The warm quick benchmark took 3.38 s with 2,000 blocks and 20 project switches. The
unchanged full workload took 8.44 s with 20,000 blocks and 200 switches. Both met the
local callback timing limit. The CI comparison script ran the base and head on this
machine and produced a report with no dropped hits. The CI run time remains unknown until
the PR runs on GitHub.
