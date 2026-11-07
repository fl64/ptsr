## ADDED Requirements

### Requirement: Working profiles exclude examples

The profile loader SHALL read only files with the `.toml` extension.
It SHALL ignore unfinished `.toml.example` files without a profile or error diagnostic.
The shipped SMC-PAD template SHALL use `.toml.example` until measured hardware data defines a working profile.
Invalid actual `.toml` files SHALL retain their file-and-line diagnostics and default device routing.

#### Scenario: Shipped profiles load without template errors
- **WHEN** the program loads the actual shipped device directory
- **THEN** all working profiles load without excluded errors
- **AND** the unfinished SMC-PAD example creates no profile or error diagnostic

#### Scenario: An example does not replace a working profile
- **WHEN** a profile directory contains a valid `.toml` and an unfinished `.toml.example`
- **THEN** the valid profile keeps its actions and the example adds no diagnostic or assignments
- **AND** a device without a valid matching profile keeps default notes 36 through 51

#### Scenario: Invalid working profiles retain default routing
- **WHEN** an actual `.toml` profile contains an invalid value
- **THEN** its diagnostic names the file and line
- **AND** its device still uses default routing without a crash
