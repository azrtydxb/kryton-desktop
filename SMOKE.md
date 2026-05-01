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
