# Git Worktree Workflow for ripVID

**A visual guide to the safe development workflow using git worktrees**

---

## Overview

This workflow keeps your `main` branch safe and stable while enabling parallel development of multiple features in isolated worktrees.

---

## Directory Structure

```
/home/user/
│
├── ripVID/                          # Main Repository (STABLE)
│   ├── .git/                        # Shared Git database
│   │   ├── hooks/
│   │   │   └── pre-commit           # Branch protection hook
│   │   └── worktrees/               # Worktree metadata
│   │
│   ├── node_modules/                # Main dependencies
│   ├── src/                         # Source code (main branch)
│   ├── dist/                        # Build output (main branch)
│   ├── package.json
│   └── ...
│
│   [Branch: main]
│   [Status: Always stable, production-ready]
│   [Commits: Only via tested PRs]
│
└── ripVID-worktrees/                # Development Area
    │
    ├── claude-audit/                # Feature: Code audit
    │   ├── node_modules/            # Independent dependencies
    │   ├── src/                     # Modified source
    │   ├── dist/                    # Independent build
    │   └── ...
    │   [Branch: claude/full-codebase-audit-011...]
    │   [Status: Active development]
    │
    ├── feature-ui-redesign/         # Feature: UI improvements
    │   ├── node_modules/
    │   ├── src/
    │   └── ...
    │   [Branch: feature-ui-redesign]
    │   [Status: Active development]
    │
    └── hotfix-security/             # Hotfix: Security issue
        ├── node_modules/
        ├── src/
        └── ...
        [Branch: hotfix-security]
        [Status: Testing]
```

---

## Workflow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     START NEW FEATURE                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │  Create Branch  │
                    │   & Worktree    │
                    └─────────────────┘
                              │
                              ▼
      ┌───────────────────────────────────────────────┐
      │  git worktree add -b feature-name             │
      │    /home/user/ripVID-worktrees/feature-name   │
      └───────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │  Install Deps   │
                    │  npm install    │
                    └─────────────────┘
                              │
                              ▼
      ┌───────────────────────────────────────────────┐
      │         DEVELOPMENT CYCLE                     │
      │  ┌─────────────────────────────────────────┐  │
      │  │  1. Edit code                           │  │
      │  │  2. Test locally (npm run tauri:dev)    │  │
      │  │  3. Commit changes                      │  │
      │  │  4. Push to remote                      │  │
      │  │  5. Repeat...                           │  │
      │  └─────────────────────────────────────────┘  │
      └───────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │  Ready to Merge │
                    └─────────────────┘
                              │
                              ▼
      ┌───────────────────────────────────────────────┐
      │         PRE-MERGE TESTING                     │
      │  ┌─────────────────────────────────────────┐  │
      │  │  ✓ Clean build (rm -rf dist target)    │  │
      │  │  ✓ Fresh install (npm install)         │  │
      │  │  ✓ Run dev server (npm run tauri:dev)  │  │
      │  │  ✓ Build production (npm run tauri:build)│ │
      │  │  ✓ Test installers                     │  │
      │  │  ✓ All tests pass                      │  │
      │  └─────────────────────────────────────────┘  │
      └───────────────────────────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │                   │
                    ▼                   ▼
           ┌────────────────┐   ┌──────────────┐
           │  Via PR        │   │  Local Merge │
           │  (Recommended) │   │  (Fast path) │
           └────────────────┘   └──────────────┘
                    │                   │
                    └─────────┬─────────┘
                              ▼
                    ┌─────────────────┐
                    │  Merge to Main  │
                    └─────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │  Clean Up       │
                    │  Remove Worktree│
                    └─────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │      DONE       │
                    └─────────────────┘
```

---

## Parallel Development

Multiple worktrees can be active simultaneously:

```
Terminal 1                    Terminal 2                    Terminal 3
────────────                  ────────────                  ────────────
$ cd ripVID-worktrees/        $ cd ripVID-worktrees/        $ cd ripVID/
  claude-audit                  feature-ui-redesign

$ npm run tauri:dev           $ npm run tauri:dev           $ npm run tauri:dev
[Running on port 5173]        [Running on port 5174]        [Running on port 5173]

Testing audit changes...      Testing UI redesign...        Testing stable main...

✓ Feature works               ✓ UI looks good               ✓ Stable version works
✓ Ready to commit             ✓ Need more work              ✓ Ready for release
```

---

## Safe Merge Process

### Option A: Pull Request (Recommended)

```
1. Push feature branch
   $ cd /home/user/ripVID-worktrees/feature-name
   $ git push -u origin feature-name

2. Create PR on GitHub/GitLab
   - Add description
   - Request reviews
   - Wait for CI/CD

3. Merge via web interface
   - Merge pull request
   - Delete branch

4. Update main locally
   $ cd /home/user/ripVID
   $ git checkout main
   $ git pull

5. Clean up worktree
   $ git worktree remove /home/user/ripVID-worktrees/feature-name
```

### Option B: Local Merge (Fast Path)

```
1. Test thoroughly in worktree
   $ cd /home/user/ripVID-worktrees/feature-name
   $ npm run build && npm run tauri:build
   [All tests pass]

2. Switch to main directory
   $ cd /home/user/ripVID
   $ git checkout main

3. Ensure main is up to date
   $ git pull origin main

4. Merge feature branch
   $ git merge --no-ff feature-name

5. Push to remote
   $ git push origin main

6. Clean up worktree
   $ git worktree remove /home/user/ripVID-worktrees/feature-name
   $ git branch -d feature-name
```

---

## Hot Fix Workflow

When production has a critical bug:

```
1. Create hotfix worktree from main
   $ cd /home/user/ripVID
   $ git worktree add -b hotfix-critical \
       /home/user/ripVID-worktrees/hotfix-critical main

2. Fix the bug
   $ cd /home/user/ripVID-worktrees/hotfix-critical
   $ npm install
   [Fix the bug...]
   $ git commit -m "fix: critical security issue"

3. Test thoroughly
   $ npm run tauri:build
   [Test the fix...]

4. Fast-track merge
   $ cd /home/user/ripVID
   $ git checkout main
   $ git merge --no-ff hotfix-critical
   $ git push origin main

5. Tag release
   $ git tag -a v2.1.2 -m "Hotfix: security fix"
   $ git push origin v2.1.2

6. Clean up
   $ git worktree remove /home/user/ripVID-worktrees/hotfix-critical
   $ git branch -d hotfix-critical
```

---

## Daily Commands

### Morning: Start Work

```bash
# Check what's active
$ git worktree list

# Go to your feature worktree
$ cd /home/user/ripVID-worktrees/claude-audit

# Pull latest changes
$ git pull

# Start development
$ npm run tauri:dev
```

### During Day: Make Changes

```bash
# Make changes, test locally
$ npm run tauri:dev

# Commit frequently
$ git add .
$ git commit -m "feat: implement feature X"

# Push to backup work
$ git push
```

### Evening: Wrap Up

```bash
# Ensure everything is committed
$ git status

# Push any unpushed commits
$ git push

# List all active worktrees
$ git worktree list

# Optional: Clean up merged branches
$ ./worktree-helper.sh clean
```

---

## Branch Protection Visual

```
/home/user/ripVID/  (main directory)
┌────────────────────────────────────────┐
│         [Branch: main]                 │
│                                        │
│   🛡️  PROTECTED BY PRE-COMMIT HOOK    │
│                                        │
│   ❌ Direct commits blocked            │
│   ✅ Only merges allowed               │
│   ✅ Always stable                     │
│                                        │
└────────────────────────────────────────┘

/home/user/ripVID-worktrees/
┌────────────────────────────────────────┐
│     [Branch: feature-xyz]              │
│                                        │
│   ✅ All commits allowed               │
│   ✅ Experimental work welcome         │
│   ✅ Test freely                       │
│                                        │
└────────────────────────────────────────┘
```

---

## Testing Strategy

```
┌─────────────────────────────────────────────────────────────┐
│                      TESTING STAGES                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. LOCAL TESTING (in worktree)                            │
│     └─ npm run tauri:dev                                   │
│     └─ Manual testing                                      │
│     └─ Quick iteration                                     │
│                                                             │
│  2. BUILD TESTING (in worktree)                            │
│     └─ npm run build                                       │
│     └─ npm run tauri:build                                 │
│     └─ Test installers                                     │
│                                                             │
│  3. COMPARISON TESTING (main vs feature)                   │
│     └─ Run both versions side-by-side                      │
│     └─ Compare behavior                                    │
│     └─ Verify improvements                                 │
│                                                             │
│  4. CLEAN BUILD TESTING (before merge)                     │
│     └─ rm -rf dist node_modules src-tauri/target          │
│     └─ npm install                                         │
│     └─ npm run tauri:build                                 │
│     └─ Full installer testing                              │
│                                                             │
│  5. POST-MERGE TESTING (in main)                           │
│     └─ Verify merge successful                             │
│     └─ Quick smoke test                                    │
│     └─ Tag release if needed                               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Quick Command Reference

```bash
# Create new feature worktree
$ ./worktree-helper.sh new feature-name

# List all worktrees
$ ./worktree-helper.sh list

# See detailed status
$ ./worktree-helper.sh status

# Remove worktree
$ ./worktree-helper.sh remove feature-name

# Clean up merged branches
$ ./worktree-helper.sh clean

# Manual commands
$ git worktree add -b feature /home/user/ripVID-worktrees/feature
$ git worktree list
$ git worktree remove /home/user/ripVID-worktrees/feature
$ git worktree prune
```

---

## Best Practices Checklist

Before creating a worktree:
- [ ] Clear purpose for the worktree
- [ ] Descriptive branch name
- [ ] Main branch is up to date

While developing in worktree:
- [ ] Commit frequently
- [ ] Push to backup work
- [ ] Test locally before pushing
- [ ] Keep worktree up to date with main

Before merging:
- [ ] All features implemented
- [ ] Clean build successful
- [ ] All tests pass
- [ ] No uncommitted changes
- [ ] Branch up to date with main
- [ ] Production build tested

After merging:
- [ ] Worktree removed
- [ ] Branch deleted (if no longer needed)
- [ ] Main branch pulled
- [ ] Tag release (if applicable)

Maintenance:
- [ ] Weekly: Review active worktrees
- [ ] Monthly: Clean up stale references
- [ ] Quarterly: Review workflow effectiveness

---

## Disk Space Management

### Typical Worktree Sizes (ripVID)

```
Component                     Size
──────────────────────────────────────
node_modules/                 ~500 MB
src-tauri/target/ (debug)     ~2 GB
src-tauri/target/ (release)   ~5 GB
dist/                         ~50 MB
──────────────────────────────────────
Total per worktree:           ~2-5 GB
```

### Cleanup Commands

```bash
# Remove build artifacts from inactive worktree
$ cd /home/user/ripVID-worktrees/inactive-feature
$ rm -rf src-tauri/target dist

# Remove old worktree completely
$ ./worktree-helper.sh remove inactive-feature

# Check disk usage
$ du -sh /home/user/ripVID-worktrees/*

# Remove all merged worktrees
$ ./worktree-helper.sh clean
```

---

## Troubleshooting Flowchart

```
Problem?
  │
  ├─ Can't create worktree for branch
  │  └─ Is branch already checked out elsewhere?
  │     ├─ Yes → Use different branch name
  │     └─ No → Check git worktree list
  │
  ├─ Port already in use
  │  └─ Change port in vite.config.ts
  │     └─ server: { port: 5174 }
  │
  ├─ Build fails in worktree
  │  └─ Dependencies installed?
  │     ├─ No → npm install
  │     └─ Yes → rm -rf node_modules && npm install
  │
  ├─ Worktree manually deleted
  │  └─ git worktree prune
  │
  ├─ Can't commit to main
  │  └─ Branch protection hook working! ✓
  │     └─ Use worktree for development
  │
  └─ Everything broken
     └─ git worktree list
        └─ git worktree repair
           └─ Or re-clone repository
```

---

## Migration Guide

### From Current State to Worktree Workflow

```
Current State:
  /home/user/ripVID/
    [claude/full-codebase-audit-011...]

Step 1: Run setup script
  $ cd /home/user/ripVID
  $ ./setup-worktrees.sh

Step 2: Verify worktree created
  $ git worktree list

Step 3: Work in worktree
  $ cd /home/user/ripVID-worktrees/claude-audit
  $ npm run tauri:dev

Step 4: Switch main to main branch
  $ cd /home/user/ripVID
  $ git checkout main  # Or create if doesn't exist

Final State:
  /home/user/ripVID/               [main]
  /home/user/ripVID-worktrees/
    └── claude-audit/              [claude/full-codebase-audit-011...]
```

---

## Summary

**Golden Rules:**
1. Main directory = main branch (always stable)
2. All development in worktrees (one per feature)
3. Test thoroughly before merging
4. Clean up after merging
5. Never push directly to main

**Key Benefits:**
- Safe parallel development
- No rebuild overhead
- Easy comparison testing
- Better for Tauri's slow Rust builds
- Protected main branch

**Resources:**
- Full guide: `GIT_WORKTREES_GUIDE.md`
- Quick reference: `WORKTREE_QUICK_REFERENCE.md`
- Setup script: `./setup-worktrees.sh`
- Helper script: `./worktree-helper.sh`

---

**Start now:** `./setup-worktrees.sh`
