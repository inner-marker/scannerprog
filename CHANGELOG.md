# Changelog

## [Unreleased] - Refactoring for Multiple Models

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