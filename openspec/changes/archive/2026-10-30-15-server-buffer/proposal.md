# 15 — Buffer size follows the server

## Why

On every start the program asks the sound server to change its buffer size (the config's `buffer_size`). The buffer is not a per-program setting: it is the shared graph's timing, so each start reconfigures everything — and on Bluetooth headphones the reopened link sometimes comes back silently dead, killing the owner's music. It happened in practice.

## What changes

- **At start the program takes the server's buffer size as is** — no request, no reconfiguration of the owner's audio graph. This is the default and what a fresh config gets (`buffer_size = 0`).
- An **explicit size still works**: `buffer_size` in the config or a choice on the settings screen is asked from the server as today.
- The settings screen's buffer row learns **1024** (the graph's usual size today) next to 64, 128, 256, 512; picking a size saves it as an explicit choice, and 0 is shown as "auto".

## Out of scope

- Latency warnings per device; JACK-only specifics of quantum negotiation.

## How to verify

1. Music plays in the browser; start the program with `buffer_size = 0` (or the line removed): the music does not blink, the status line shows the server's size (e.g. 1024).
2. Restart the program a few times: the sound never drops.
3. F12 → pick 256: the size changes as before and is saved; setting it back to auto returns to the server's size after a restart.
