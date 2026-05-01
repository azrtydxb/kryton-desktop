# Smoke Testing Checklist

Manual smoke testing for Kryton desktop app. Run these checks after a successful build to verify core functionality.

## Pre-test Setup

- [ ] Build the app: `npm run build`
- [ ] Start the dev server: `npm run vite:dev` (in one terminal)
- [ ] Run Tauri dev: `npm run dev` (in another terminal)

## Core UI

- [ ] App launches without errors
- [ ] Main window appears and is responsive
- [ ] Tailwind CSS styling is applied correctly
- [ ] Responsive layout works at different window sizes

## Authentication & First Run

- [ ] First-run flow displays correctly
- [ ] LoginScreen renders without errors
- [ ] ServerSetup flow is accessible
- [ ] Auth token storage works (verify in app data)

## File Dialogs

- [ ] File picker dialog opens when triggered
- [ ] File selection works and returns correct path
- [ ] Dialog cancel works without errors

## Native Menu Bar

- [ ] Native menu bar appears (platform-specific)
- [ ] Menu items are clickable
- [ ] Menu actions execute correctly

## Tauri Integration

- [ ] Window state persists (maximize/minimize/size)
- [ ] App can be quit gracefully
- [ ] No console errors in dev tools
- [ ] IPC commands execute without errors

## Core Functionality

- [ ] Database adapter initializes correctly
- [ ] Data persistence works
- [ ] No TypeScript compilation errors
- [ ] All unit tests pass: `npm run test`

## Build & Artifacts

- [ ] Debug build completes successfully
- [ ] Build artifacts are created in expected locations
- [ ] No warnings during build process

## Dev Linking

- [ ] `npm run dev:link` completes without errors
- [ ] Dev link verification passes: `npm run dev:verify`
- [ ] `npm run dev:unlink` works correctly

## Tier 3 Native Integrations (Phase B)

- [ ] Native notification appears when `notify(title, body)` is called from the frontend (requires permission grant on first use)
- [ ] Spotlight: after calling `spotlight_index`, note title appears in macOS Spotlight search (⌘Space)
- [ ] Spotlight: `spotlight_remove` removes a previously-indexed note from results
- [ ] Windows Search: `.url` shortcut files appear in `%LocalAppData%\Microsoft\Windows\Start Menu\Programs\Kryton\<account_id>\` after `win_search_upsert_note`
- [ ] Windows taskbar jumplist: right-clicking the Kryton taskbar icon shows "Recent Notes" and tasks (New Note, Open Quick Switcher)

## Known Limitations

### DCC-B4: Touch Bar — deferred to v1.1

The `objc2-app-kit` crate (v0.3.2) exposes `NSTouchBar` bindings, but implementing a functional Touch Bar requires defining a custom Objective-C delegate class via `define_class!` and managing `MainThreadOnly` objects across Tauri's async runtime boundary. The integration risk outweighs the benefit for v1 (Touch Bar hardware was discontinued by Apple in 2021 with the 14/16-inch MacBook Pro lineup). Deferred per spec-sanctioned skip policy.

**Re-enable in v1.1:** Implement `touchbar.rs` with a `define_class!`-based `NSTouchBarDelegate`, set it on the `NSWindowController` for each account window, and expose a Tauri event listener to update the current note title and sync status in real time.
