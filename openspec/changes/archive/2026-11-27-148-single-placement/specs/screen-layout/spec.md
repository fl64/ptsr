## ADDED Requirements

### Requirement: The visible mode has its badge

Placement badges SHALL use `IN HAND`, `COPY`, `SWAP` or `ASSIGN`.
Marking SHALL show `MARK` or `N MARKED`. Edit SHALL show `EDIT` with its page.
Learning and questions SHALL use `LEARN` and `ASK`. Recording SHALL show `REC`
with elapsed time if it fits. Chop toggles SHALL show `LOOP` and `AS PAD`.
Marks and Assign SHALL retain their counts. A badge SHALL show only an active mode.
Transport and pad playback SHALL show separate feedback.

#### Scenario: Every existing mode has its badge
- **WHEN** each existing placement, marking, edit, learning, question, recording or Chop toggle state is active in a separate fixture
- **THEN** its badge uses the specified label, page, count or elapsed time
- **AND** transport feedback remains separate from pad playback

#### Scenario: One placement badge names the held item
- **WHEN** a successful pickup replaces an item in hand
- **THEN** the badge names only the new item

## REMOVED Requirements

### Requirement: Existing mode badge labels

**Reason**: One item can wait for placement. The compound placement badge has
no active state.

**Migration**: Use the visible mode badge requirement for the current item.
