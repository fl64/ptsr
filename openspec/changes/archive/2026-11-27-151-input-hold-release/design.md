## Event order

A folder picker receives new actions. A release from an earlier keyboard Gate still goes to that Gate. A new pad press in the picker stays silent.

The keyboard keeps the key that started the microphone take. A later release matches that key without modifiers. A release for an old take does not stop a newer take.

The RECORD release ends a microphone take without answering the question. A release cannot end an output recording.

## Verification cases

| Sequence | Expected result | Check |
| --- | --- | --- |
| Hold a Gate, change banks, open a folder picker, then release | The original pad stops. The picker stays open. | Offline audio and keyboard events |
| Press and release a new pad key in the picker | The pad remains silent. | Keyboard events |
| Start a microphone hold and open a question | The owning release stops the take. The question holds its state. | Keyboard and recorded MIDI |
| Release Alt before R, or R before Alt | Both orders stop the same take. | Keyboard events |
| Press or release RECORD while a keyboard take runs | The keyboard take continues. | Recorded MIDI and keyboard events |
| Use a terminal without key releases | Two key presses start and stop. | Legacy keyboard events |
| A take ends before its held control releases | The release cannot stop a later take. | Match the take start time |

The physical controller and microphone listening checks stay open.
