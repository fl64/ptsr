# Tasks — 15

- [x] 1. Config: `buffer_size` default 0 (auto), 0 round-trips through load/save; default-config test; commit.
- [x] 2. Audio: `start_jack` and the cpal path skip the size request when it is 0 and take the server's/device's size; log line names the size and "auto"; unit tests on the decision helper; commit.
- [x] 3. Settings screen: the buffer row cycles auto…1024, shows "auto (N)" with the actual size and one line explaining when to change it; status line says auto for 0; tests; commit.
