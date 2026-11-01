# Scenarios without an automatic test

The test `crates/app/tests/scenarios.rs` fails when a scenario of `openspec/specs/` has no test that names it. A scenario on this list has no test, for the reason given. Each line is: a scenario name in backticks, its spec in brackets, the reason. A line for a scenario that is gone, or that has a test now, also fails the test.

- `Sound server unavailable` (audio-engine): the logic is in `audio::start` and `main`; a test start would connect to the sound server of the owner. The text of the message has the test `audio_trouble_is_said_in_plain_words_once`.
- `Start does not touch the graph` (audio-settings): the condition is inside `start_jack` and needs a real JACK client. The config part has the test `a_fresh_config_follows_the_server_buffer`.
- `An explicit size still applies` (audio-settings): the same place, `start_jack`, needs a real JACK client.
- `Server did not accept the buffer` (audio-settings): `App::change` calls `Audio::set_buffer_size` on a real client; `App.audio` has no stand-in.
