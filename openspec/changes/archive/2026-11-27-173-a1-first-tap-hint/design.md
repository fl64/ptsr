## Decision

Use the current tap timestamps for the cue. Show the keyboard cue in the notice row.
Use the existing mode line after controller input. Give warnings and errors priority.

| State | Result |
| --- | --- |
| First tap | Keep the tempo and show the cue. |
| Tap less than 200 ms later | Keep the first tap and its deadline. |
| Second accepted tap | Remove the cue and show BPM. |
| No second tap within two seconds | Remove the cue. |
| New sequence after a tempo result | Show the new cue in place of the earlier result. |
| Help, a question, or an open text input | Hide the keyboard cue. |

## Open questions

Use the current cue text for both inputs. The owner gave no text preference.
