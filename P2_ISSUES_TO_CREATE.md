# P2+ Issues to Create on GitHub

Generated from comprehensive audit on 2025-11-05
Based on: MASTER_AUDIT_REPORT.md

---

## 🔴 P2 - HIGH PRIORITY (Week 2)

### Issue 1: Install ESLint for Code Quality
**Priority:** P2 - HIGH
**Estimated Time:** 30 minutes
**Labels:** `enhancement`, `tooling`, `code-quality`

**Description:**
Install and configure ESLint with TypeScript support to enforce code quality standards.

**Tasks:**
- [ ] Install ESLint and plugins: `@typescript-eslint/eslint-plugin`, `@typescript-eslint/parser`, `eslint-plugin-react`, `eslint-plugin-react-hooks`, `eslint-config-prettier`
- [ ] Create `.eslintrc.json` configuration
- [ ] Add `"lint": "eslint src --ext ts,tsx"` script to package.json
- [ ] Run `bun run lint --fix` to fix auto-fixable issues
- [ ] Update CI/CD to run linting

**References:**
- MASTER_AUDIT_REPORT.md line 351-358
- BUILD_CONFIGURATION_AUDIT.md

---

### Issue 2: Install Prettier for Code Formatting
**Priority:** P2 - HIGH
**Estimated Time:** 30 minutes
**Labels:** `enhancement`, `tooling`, `code-quality`

**Description:**
Install and configure Prettier to enforce consistent code formatting across the codebase.

**Tasks:**
- [ ] Install Prettier: `bun add -D prettier`
- [ ] Create `prettier.config.js` configuration
- [ ] Create `.prettierignore` file
- [ ] Add `"format": "prettier --write src"` script to package.json
- [ ] Run `bun run format` to format all code
- [ ] Update CI/CD to check formatting

**References:**
- MASTER_AUDIT_REPORT.md line 359-369

---

### Issue 3: Add Process Timeouts for Downloads
**Priority:** P2 - HIGH
**Estimated Time:** 2 hours
**Labels:** `bug`, `backend`, `rust`, `reliability`

**Description:**
Downloads can hang indefinitely if yt-dlp process stalls. Add configurable timeouts to prevent resource exhaustion.

**Current Issue:**
- `download.rs:432-447` - No timeout on process spawn
- Risk: Zombie processes, resource leaks, poor UX

**Tasks:**
- [ ] Add timeout configuration (recommend 30 minutes default)
- [ ] Implement timeout in `execute_ytdlp_with_retry()`
- [ ] Add timeout error handling and user notification
- [ ] Emit `download-failed` event on timeout
- [ ] Add tests for timeout behavior

**Files to Modify:**
- `src-tauri/src/download.rs:432-447`
- `src-tauri/src/main.rs` (add timeout config)

**References:**
- MASTER_AUDIT_REPORT.md line 282
- Backend Issues section

---

### Issue 4: Add Concurrent Download Limits
**Priority:** P2 - MEDIUM
**Estimated Time:** 1 hour
**Labels:** `enhancement`, `backend`, `rust`, `reliability`

**Description:**
Currently unlimited concurrent downloads can cause resource exhaustion and DoS risk.

**Tasks:**
- [ ] Add `MAX_CONCURRENT_DOWNLOADS` constant (recommend 3)
- [ ] Implement queue system in AppState
- [ ] Return error when limit exceeded with clear message
- [ ] Add UI indication when download queued
- [ ] Consider adding download queue panel

**Files to Modify:**
- `src-tauri/src/main.rs:32` (AppState)
- `src-tauri/src/download.rs`

**References:**
- MASTER_AUDIT_REPORT.md line 284

---

### Issue 5: Validate localStorage JSON Parsing
**Priority:** P2 - MEDIUM
**Estimated Time:** 1 hour
**Labels:** `bug`, `frontend`, `security`, `reliability`

**Description:**
localStorage data is parsed without validation, risking crashes from corrupted data.

**Current Issue:**
```typescript
const saved = localStorage.getItem("ripvid-archive");
if (saved) {
    const loadedArchive = JSON.parse(saved); // ❌ No validation
    setArchive(loadedArchive);
}
```

**Tasks:**
- [ ] Add Zod or similar validation library
- [ ] Create schema for ArchiveItem
- [ ] Wrap JSON.parse in try-catch
- [ ] Validate parsed data against schema
- [ ] Clear corrupted data and log error
- [ ] Apply to all localStorage usage

**Files to Modify:**
- `src/App.tsx:218-224` (archive)
- `src/App.tsx:226-229` (format)
- `src/App.tsx:231-234` (quality)

**References:**
- MASTER_AUDIT_REPORT.md line 271
- SEC-005 in security audit

---

### Issue 6: Extract Magic Strings to Constants
**Priority:** P2 - MEDIUM
**Estimated Time:** 2 hours
**Labels:** `refactor`, `frontend`, `code-quality`

**Description:**
Magic strings scattered throughout App.tsx make code hard to maintain and error-prone.

**Current Issues:**
- Event names: `"download-progress"`, `"download-complete"`, etc.
- localStorage keys: `"ripvid-archive"`, `"ripvid-format"`, etc.
- Platform names: `"youtube"`, `"x"`, `"facebook"`, etc.

**Tasks:**
- [ ] Create `src/constants/index.ts`
- [ ] Extract event names to `EVENTS` object
- [ ] Extract localStorage keys to `STORAGE_KEYS` object
- [ ] Extract platform names to `PLATFORMS` object
- [ ] Replace all magic strings with constants
- [ ] Add JSDoc comments explaining each constant

**Files to Create:**
- `src/constants/index.ts`

**Files to Modify:**
- `src/App.tsx` (replace ~30 magic strings)

**References:**
- MASTER_AUDIT_REPORT.md line 272

---

### Issue 7: Add TypeScript Path Aliases
**Priority:** P2 - MEDIUM
**Estimated Time:** 15 minutes
**Labels:** `enhancement`, `tooling`, `dx`

**Description:**
Add path aliases to avoid messy relative imports like `../../../components`.

**Tasks:**
- [ ] Add paths to `tsconfig.json`:
  ```json
  {
    "compilerOptions": {
      "paths": {
        "@/*": ["./src/*"],
        "@components/*": ["./src/components/*"],
        "@hooks/*": ["./src/hooks/*"],
        "@utils/*": ["./src/utils/*"]
      }
    }
  }
  ```
- [ ] Update Vite config to resolve aliases
- [ ] Update existing imports (only 2-3 files currently)

**Files to Modify:**
- `tsconfig.json`
- `vite.config.ts`

**References:**
- MASTER_AUDIT_REPORT.md line 299

---

### Issue 8: Implement Security Scanning in CI/CD
**Priority:** P2 - HIGH
**Estimated Time:** 30 minutes
**Labels:** `security`, `ci-cd`, `devops`

**Description:**
Security scan in CI/CD is currently a stub: `echo "Running security audit..."`

**Tasks:**
- [ ] Add `cargo audit` for Rust dependencies
- [ ] Add `npm audit` (or `bun audit`) for frontend
- [ ] Generate SBOM (Software Bill of Materials)
- [ ] Fail build on high/critical vulnerabilities
- [ ] Add security scan badge to README

**Files to Modify:**
- `.github/workflows/desktop-release.yml`
- `.github/workflows/security-audit.yml` (new file)

**References:**
- MASTER_AUDIT_REPORT.md line 298
- Build System audit

---

## 🟡 P3 - MEDIUM PRIORITY (Week 3-4)

### Issue 9: Refactor Monolithic App.tsx
**Priority:** P3 - MEDIUM (High impact but time-consuming)
**Estimated Time:** 8 hours
**Labels:** `refactor`, `frontend`, `architecture`

**Description:**
App.tsx is 871 lines (4.3x over 200 line recommendation) with 13 useState hooks. Break into smaller components.

**Proposed Architecture:**
```
src/
├── components/
│   ├── DownloadPanel.tsx        (input, button, progress)
│   ├── ArchivePanel.tsx         (history list)
│   ├── SettingsPanel.tsx        (quality, format settings)
│   └── ...
├── hooks/
│   ├── useDownload.ts           (download logic)
│   ├── useArchive.ts            (archive management)
│   └── useDownloadEvents.ts     (Tauri event listeners)
└── App.tsx                      (< 200 lines, composition)
```

**Tasks:**
- [ ] Extract DownloadPanel component (input + button + status)
- [ ] Extract ArchivePanel component (archive list + tabs)
- [ ] Extract SettingsPanel component (quality + format)
- [ ] Create useDownload custom hook
- [ ] Create useArchive custom hook
- [ ] Create useDownloadEvents custom hook
- [ ] Update App.tsx to compose components
- [ ] Verify all functionality works

**Files to Create:**
- `src/components/DownloadPanel.tsx`
- `src/components/ArchivePanel.tsx`
- `src/components/SettingsPanel.tsx`
- `src/hooks/useDownload.ts`
- `src/hooks/useArchive.ts`
- `src/hooks/useDownloadEvents.ts`

**Files to Modify:**
- `src/App.tsx` (reduce from 871 to ~150 lines)

**References:**
- MASTER_AUDIT_REPORT.md line 268
- FRONTEND_AUDIT_REPORT.md

---

### Issue 10: Add Error Boundaries
**Priority:** P3 - MEDIUM
**Estimated Time:** 2 hours
**Labels:** `enhancement`, `frontend`, `reliability`

**Description:**
No error boundaries means entire app crashes on component errors. Add graceful error handling.

**Tasks:**
- [ ] Create `ErrorBoundary.tsx` component
- [ ] Add fallback UI with error message
- [ ] Add "Reload" and "Report Issue" buttons
- [ ] Wrap App with ErrorBoundary
- [ ] Add boundaries around ShaderBackground
- [ ] Log errors to console for debugging

**Files to Create:**
- `src/components/ErrorBoundary.tsx`

**Files to Modify:**
- `src/main.tsx` (wrap App)
- `src/App.tsx` (wrap ShaderBackground)

**References:**
- MASTER_AUDIT_REPORT.md line 273

---

### Issue 11: Fix Silent Event Failures
**Priority:** P3 - MEDIUM
**Estimated Time:** 2 hours
**Labels:** `bug`, `backend`, `rust`, `reliability`

**Description:**
Tauri event emissions fail silently if no listeners are attached, making debugging difficult.

**Tasks:**
- [ ] Add error handling for all `emit()` calls
- [ ] Log warnings when events fail to emit
- [ ] Consider retry mechanism for critical events
- [ ] Add event emission metrics

**Files to Modify:**
- `src-tauri/src/download.rs` (multiple emit calls)

**References:**
- MASTER_AUDIT_REPORT.md line 283

---

### Issue 12: Deduplicate SHA256 Verification Code
**Priority:** P3 - MEDIUM
**Estimated Time:** 3 hours
**Labels:** `refactor`, `backend`, `rust`, `code-quality`

**Description:**
SHA256 verification is duplicated in `ytdlp_updater.rs` and `binary_manager.rs` (~200 lines).

**Tasks:**
- [ ] Create `src-tauri/src/utils/crypto.rs`
- [ ] Extract `verify_checksum()` function
- [ ] Extract `download_with_progress()` function
- [ ] Update YtdlpUpdater to use utils
- [ ] Update BinaryManager to use utils
- [ ] Add unit tests for crypto utils

**Files to Create:**
- `src-tauri/src/utils/crypto.rs`
- `src-tauri/src/utils/mod.rs`

**Files to Modify:**
- `src-tauri/src/ytdlp_updater.rs`
- `src-tauri/src/binary_manager.rs`
- `src-tauri/src/main.rs` (add utils module)

**References:**
- MASTER_AUDIT_REPORT.md line 285

---

### Issue 13: Add Download Size Limits
**Priority:** P3 - LOW
**Estimated Time:** 1 hour
**Labels:** `enhancement`, `security`, `backend`, `rust`

**Description:**
No limits on download size could lead to disk space exhaustion.

**Tasks:**
- [ ] Add `MAX_DOWNLOAD_SIZE_MB` config (recommend 500 MB)
- [ ] Check Content-Length header before download
- [ ] Monitor size during download
- [ ] Cancel if limit exceeded
- [ ] Show clear error message to user

**Files to Modify:**
- `src-tauri/src/download.rs`

**References:**
- MASTER_AUDIT_REPORT.md line 87
- SEC-004 in security audit

---

### Issue 14: Optimize ShaderBackground Performance
**Priority:** P3 - MEDIUM
**Estimated Time:** 2 hours
**Labels:** `performance`, `frontend`, `optimization`

**Description:**
ShaderBackground animates constantly even when app is idle, wasting GPU/CPU.

**Tasks:**
- [ ] Add `requestAnimationFrame` throttling
- [ ] Pause animation when app minimized
- [ ] Add user preference to disable animations
- [ ] Reduce animation complexity for low-end devices
- [ ] Add performance monitoring

**Files to Modify:**
- `src/components/ShaderBackground.tsx`

**References:**
- MASTER_AUDIT_REPORT.md line 275

---

## 🟢 P4 - LOW PRIORITY (Month 2+)

### Issue 15: Remove Legacy YtdlpUpdater
**Priority:** P4 - LOW
**Estimated Time:** 4 hours
**Labels:** `refactor`, `backend`, `rust`, `cleanup`

**Description:**
BinaryManager replaces YtdlpUpdater, but old code still exists. Complete migration.

**Tasks:**
- [ ] Verify BinaryManager handles all yt-dlp updates
- [ ] Remove YtdlpUpdater from AppState
- [ ] Delete `src-tauri/src/ytdlp_updater.rs`
- [ ] Update all references
- [ ] Test yt-dlp updates still work

**Files to Delete:**
- `src-tauri/src/ytdlp_updater.rs`

**Files to Modify:**
- `src-tauri/src/main.rs`

**References:**
- MASTER_AUDIT_REPORT.md line 286

---

### Issue 16: Remove Unused Dependencies
**Priority:** P4 - LOW
**Estimated Time:** 30 minutes
**Labels:** `maintenance`, `dependencies`, `optimization`

**Description:**
8 unused packages waste ~30-40 MB of node_modules space.

**Unused Packages:**
- @radix-ui/react-slot
- class-variance-authority
- clsx
- tailwind-merge
- (others identified in audit)

**Tasks:**
- [ ] Verify each package is truly unused
- [ ] Remove from package.json
- [ ] Run `bun install` to clean up
- [ ] Test app still works
- [ ] Update lock file

**References:**
- MASTER_AUDIT_REPORT.md line 150-156

---

### Issue 17: Fix Dependency Version Mismatches
**Priority:** P4 - LOW
**Estimated Time:** 10 minutes
**Labels:** `bug`, `dependencies`, `maintenance`

**Description:**
Version mismatches between package files cause confusion.

**Current State:**
- package.json: `2.1.0`
- bun.lock: `2.0.0`

**Tasks:**
- [ ] Sync all version numbers to 2.1.1
- [ ] Run `bun install` to update lock file
- [ ] Commit updated files

**References:**
- MASTER_AUDIT_REPORT.md line 137-141

---

### Issue 18: Add Type Definitions File
**Priority:** P4 - LOW
**Estimated Time:** 1 hour
**Labels:** `refactor`, `frontend`, `typescript`

**Description:**
Type definitions are duplicated across components. Centralize in types file.

**Tasks:**
- [ ] Create `src/types/index.ts`
- [ ] Move DownloadProgress interface
- [ ] Move DownloadStarted interface
- [ ] Move ArchiveItem interface
- [ ] Export all types
- [ ] Update imports across codebase

**Files to Create:**
- `src/types/index.ts`

**Files to Modify:**
- `src/App.tsx` (remove type definitions)

**References:**
- MASTER_AUDIT_REPORT.md line 274

---

## 🧪 TESTING (P5 - Ongoing)

### Issue 19: Setup Testing Infrastructure
**Priority:** P5 - Future Work
**Estimated Time:** 8 hours
**Labels:** `testing`, `infrastructure`, `tooling`

**Description:**
Zero test coverage. Setup Vitest + React Testing Library.

**Tasks:**
- [ ] Install Vitest and dependencies
- [ ] Configure vitest.config.ts
- [ ] Setup @testing-library/react
- [ ] Create test utils and mocks
- [ ] Write example tests
- [ ] Add `test` script to package.json
- [ ] Add tests to CI/CD

**Goal:** 50%+ coverage

**References:**
- MASTER_AUDIT_REPORT.md line 270, 387-391

---

### Issue 20: Add Frontend Unit Tests
**Priority:** P5 - Future Work
**Estimated Time:** 16 hours
**Labels:** `testing`, `frontend`

**Description:**
Write comprehensive unit tests for frontend components and hooks.

**Test Coverage Needed:**
- [ ] Custom hooks (useDownload, useArchive, useDownloadEvents)
- [ ] Pure functions (detectPlatform, getDownloadPath)
- [ ] Component rendering (DownloadPanel, ArchivePanel, SettingsPanel)
- [ ] Event handlers
- [ ] localStorage utilities

**Goal:** 80%+ coverage

**References:**
- MASTER_AUDIT_REPORT.md line 270

---

### Issue 21: Add Backend Unit Tests
**Priority:** P5 - Future Work
**Estimated Time:** 16 hours
**Labels:** `testing`, `backend`, `rust`

**Description:**
Add Rust unit tests for backend logic.

**Test Coverage Needed:**
- [ ] validation.rs (already has some tests)
- [ ] download.rs (download logic)
- [ ] binary_manager.rs (update logic)
- [ ] errors.rs (error handling)
- [ ] Command handlers in main.rs

**Goal:** 80%+ coverage

**References:**
- MASTER_AUDIT_REPORT.md line 287

---

## 📊 SUMMARY

**Total Issues to Create:** 21

**Priority Breakdown:**
- P2 (High - Week 2): 8 issues (~10 hours)
- P3 (Medium - Week 3-4): 7 issues (~20 hours)
- P4 (Low - Month 2+): 4 issues (~6 hours)
- P5 (Testing - Ongoing): 2 issues (~40 hours)

**Quick Wins (< 1 hour):**
- Issue 1: ESLint (30 min)
- Issue 2: Prettier (30 min)
- Issue 7: TypeScript paths (15 min)
- Issue 8: Security scanning (30 min)
- Issue 16: Remove unused deps (30 min)
- Issue 17: Version sync (10 min)

**Next Steps:**
1. Create these issues on GitHub
2. Start with P2 quick wins (Issues 1, 2, 7, 8)
3. Move to P2 high-impact items (Issues 3, 4, 5)
4. Continue with P3 refactoring work

---

**Generated:** 2025-11-05
**Based on:** MASTER_AUDIT_REPORT.md comprehensive audit
