# Changelog

## [0.2.1] - Refined Scanner Interface

- Scanner page rebuilt per `Roadmaps/0-2-1 SCANNER_PAGE_PLAN.md` (Phase 1, XT/P2 family: BCD325P2, BCD396XT, BC346XT):
  - Capability-driven controls: every control is always visible; ones the model can't use are dimmed (⛔), ones that need Program Mode show 🔒, and clicking a blocked control explains why.
  - Connection bar with health states (connecting, not responding, lost), a mode bar (Scan, Service, Custom, Close Call, Quick Search, Weather, Tone-Out), Now Playing with signal bars, transport, volume/squelch/mute, quick tune, jump to tag, an activity log with CSV export, a TX/RX log drawer, and a status strip.
  - Keyboard shortcuts: Space (hold), ←/→, A (avoid), S (scan), M (mute).
  - Polls tolerate dropped replies; the link is only dropped after ten failures in a row.

- Added the BCD996XT and BCD996P2 (database and live control; untested on hardware). Their volume (0–29) and squelch (0–19) ranges differ, there is no power-off command, and the `[RECORD]` fields are not preserved.
- Scanner page Phase 2:
  - Added staged System/Group Quick Key edits with a single Program Mode apply, discard support, paging, digit shortcuts, and a visible Program Mode badge.
  - Added a demand-driven `STS` LCD mirror, model-specific virtual keypads with tap/long-press gestures, `CSC` sweep display, battery/weather/GPS status, and log/error navigation.
  - Added live-control profiles for the BR330T, BCD996T, BCT15 and BCT15X. Their older database dialects remain explicitly unsupported rather than being sent incompatible XT/P2 memory commands.
  - Added parsers/tests for `STS` and streamed `CSC` samples. Hardware transcript/UI replay remains pending captured data.
- Scanner page Phase 3:
  - Added live-control profiles for BCD436HP, BCD536HP, SDS100, SDS150/SDS150GBT, and SDS200. Database transfer remains explicitly unavailable for these Favorites List database formats.
  - Added multiline `GSI` XML transport and normalized HomePatrol/SDS state, including transient system/department/channel handles, hold state, Favorites List context, volume/squelch, signal, battery, mute, and recording state.
  - Added native system/department/channel hold, next/previous, temporary/permanent avoid, scan resume, quick tune, three-part number-tag jump, and `URC` recording control with documented error messages.
  - Added live FQK/SQK/DQK quick-key grids, detailed SDS `GCS` battery status, and a bounded SDS `GST`/`GWF` waterfall display.
  - `GSI` and `GWF` are polled because the supplied specifications omit PSI/PWF activation details. Hardware transcript/UI replay remains pending captured data.

## [0.2.0] - Basic Scanner Functions

TThis update adds the ability to control the scanner functions from the app.

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