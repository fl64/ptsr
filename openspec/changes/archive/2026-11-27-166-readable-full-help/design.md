## Decision

Full Help uses the full work area. Its bottom border represents the Help command panel. This avoids a second grid over the text. The existing controls keep their roles. The footer names the second bank when the hold is off or unavailable. A question above Help keeps its own answer controls.

| State | Expected result |
| --- | --- |
| Panel visible before F1 | Help stays readable. Back restores the panel and the work screen. |
| Panel opens while Help is visible | The help border names the same commands. The grid waits behind Help. |
| Help, Messages or Controller | P1 closes. P5 and P6 page, scroll or select. LEFT and RIGHT change tabs. |
| Controller with keyboard input | Show the selection arrows at both sizes. |
| Hold off or unavailable | Use the second pad bank. |
| Question above Help | Answer the question first. Help keeps its place. |
| Controller disconnect | Keyboard Help remains available. |

The owner requested a manual pass for each interface or behavior change. For this change, use both terminal sizes, the keyboard and raw MIDI. Use the regular application on a separate terminal and with audio disconnected.
