## Decisions

Only a successful pickup replaces the item in hand. Keep the existing target
rules for copy, swap and file placement.

Accept a swap source that has an assigned file and is ready. Accept an empty
pad as a target. A folder or absent file selection does not replace the hand.

A successful recording save uses the same file pickup rule. Dismiss the old
placement question when the saved recording replaces its item.

## Edge cases

- Copy then swap and swap then copy. Both orders change one target at a time.
- A file replaces a copy. Both devices put the file on the target.
- An empty, missing or loading pad cannot replace the previous item.
- A recording completes while a replacement question names an old item.
- Undo and the next pad press use the result of the last placement.

## Open questions

None.
