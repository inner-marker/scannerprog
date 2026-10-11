# Changelog

## [0.1.3] - Common Database

- Bump Rust edition to 2024
- Cargo.toml updated with 0.1.3

The database is now a single common database shared by every supported scanner model.

- The database is no longer tied to one model. It stays fully editable when the connected scanner cannot use all of it; a banner, a warning message and per-system badges show what is incompatible.
- Uploading sends only the part of the database the target scanner can use (supported system types, frequency coverage, DMR settings, memory limits). The confirmation lists what will be skipped or changed, and the database itself is never altered.
- With no scanner connected, the target model can be chosen on the Database page.
- Downloading merges into the database instead of replacing it. Unmodified systems are updated, systems edited locally are kept, and new systems are added. Each system records where it came from.
- Saved files gain a format version, system identifiers and provenance. Older files load unchanged and are upgraded automatically.
- Groundwork for importing from other sources (such as RadioReference) later.

## [0.1.2] - Refactoring for Multiple Models

This update primarily focuses on refactoring the codebase to support multiple Uniden scanner models.

At this initial stage, there are now three models supported:

- Uniden BCD325P2
- Uniden BCD396XT ‼️
- Uniden BC346XT ‼️

‼️ = Untested with a Physical Device

### Additional Changes

- Add comments throughout the codebase for better readability and maintainability

## [0.1.1] - Build Correction

 - Fix release.yml Github Actions workflow (Dioxus CLI version updated to 0.7.10)

## [0.1.0] - Initial Release

- [x] Detect a Uniden scanner
- [x] Read the basic scanner information
- [x] Read the database from the scanner
- [x] Save/Load database to/from a `.ron` file
- [x] Write the database to the scanner
- [x] Message center for validation, file and transfer status, and other messages
- [x] Confirmation dialog (with optional detailed explanation) for Upload, Load, Reset and delete buttons