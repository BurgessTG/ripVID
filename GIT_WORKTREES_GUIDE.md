# Git Worktrees Guide for ripVID

**A comprehensive guide to setting up and using git worktrees for safe parallel development**

---

## Table of Contents

1. [What Are Git Worktrees?](#what-are-git-worktrees)
2. [Benefits for Your Workflow](#benefits-for-your-workflow)
3. [When to Use Worktrees](#when-to-use-worktrees)
4. [Directory Structure](#directory-structure)
5. [Step-by-Step Setup](#step-by-step-setup)
6. [Daily Workflow](#daily-workflow)
7. [Testing in Worktrees](#testing-in-worktrees)
8. [Best Practices](#best-practices)
9. [Common Commands Reference](#common-commands-reference)
10. [Common Pitfalls & Troubleshooting](#common-pitfalls--troubleshooting)
11. [Integration with ripVID Workflow](#integration-with-ripvid-workflow)

---

## What Are Git Worktrees?

Git worktrees allow you to have **multiple working directories** attached to the same Git repository. Instead of constantly switching branches (which can disrupt your build artifacts and running processes), you can have different branches checked out in different directories simultaneously.

### Traditional Workflow (Single Working Directory)
```
/home/user/ripVID/  (switches between branches)
  - Switch to feature-branch → build → test
  - Switch to main → build → test
  - Switch back to feature-branch → rebuild everything
```

### Worktree Workflow (Multiple Working Directories)
```
/home/user/ripVID/              (main branch - stable)
/home/user/ripVID-worktrees/
  ├── feature-auth/             (feature branch)
  ├── bugfix-download/          (bugfix branch)
  └── claude-audit/             (current audit branch)
```

Each directory maintains its own:
- Working files
- Build artifacts (`dist/`, `src-tauri/target/`)
- Dependencies (`node_modules/`)
- Running processes (dev servers, tests)

But they all share:
- Git history (`.git/` directory)
- Commits, branches, and refs

---

## Benefits for Your Workflow

### 1. Safe Parallel Development
- Keep main branch stable and untouched in the main directory
- Develop features in separate worktrees without affecting main
- Test thoroughly before merging
- No accidental commits to main

### 2. Eliminate Rebuild Overhead
- No need to rebuild when switching contexts
- Build artifacts persist in each worktree
- Fast context switching between tasks

### 3. Easy Comparison
- Run old and new versions side-by-side
- Compare behavior across branches
- Test backward compatibility

### 4. Risk Mitigation
- Main branch stays clean and deployable
- Experimental work stays isolated
- Easy to abandon failed experiments without cleanup

### 5. Better for Tauri Development
- Tauri builds are slow (Rust compilation)
- Keep production builds separate from development
- Test installers without disrupting dev environment

---

## When to Use Worktrees

### Use Worktrees For:
- Long-running feature branches (like your current audit branch)
- Maintaining a stable main branch while developing
- Hot fixes that need immediate testing against production
- Comparing behavior between branches
- Code review (checkout PR in separate worktree)

### Don't Need Worktrees For:
- Quick branch switches with no build needed
- Branches you'll only visit once
- Simple file edits that don't require building

---

## Directory Structure

### Recommended Structure

```
/home/user/
├── ripVID/                          # Main worktree (stable/production)
│   ├── .git/                        # Shared git directory
│   ├── node_modules/
│   ├── dist/
│   ├── src-tauri/target/
│   └── ... (all project files)
│
└── ripVID-worktrees/                # All additional worktrees
    ├── claude-audit/                # Current feature branch
    │   ├── node_modules/            # Independent dependencies
    │   ├── dist/                    # Independent build
    │   └── src-tauri/target/        # Independent Rust build
    │
    ├── feature-ui-redesign/         # Another feature
    │   └── ...
    │
    └── hotfix-security/             # Hot fix branch
        └── ...
```

### Why This Structure?

1. **Main directory** (`/home/user/ripVID/`) stays on stable branch (main)
2. **Worktrees directory** (`/home/user/ripVID-worktrees/`) contains all development work
3. **Clear separation** between stable and experimental
4. **Easy to find** all active work in one place

---

## Step-by-Step Setup

### Initial Setup (One-Time)

#### 1. Create Main Branch (if it doesn't exist)

First, let's check if you have a main branch:

```bash
cd /home/user/ripVID
git branch -a
```

If you don't have a main/master branch, create one from your current stable state:

```bash
# If you don't have a main branch yet
git checkout -b main
git push -u origin main

# Go back to your current branch
git checkout claude/full-codebase-audit-011CUpyG3C2tZ8kuMuuqc7gs
```

#### 2. Set Up Main Worktree

Your current directory will become the main worktree:

```bash
# Navigate to your repository
cd /home/user/ripVID

# Ensure you're on main branch for the main worktree
# (Do this only when you're ready - for now, stay on your current branch)
# git checkout main
```

#### 3. Create Worktrees Directory

```bash
# Create directory for all worktrees
mkdir -p /home/user/ripVID-worktrees
```

#### 4. Create Your First Worktree (Current Feature Branch)

```bash
cd /home/user/ripVID

# Create worktree for your current audit branch
git worktree add /home/user/ripVID-worktrees/claude-audit claude/full-codebase-audit-011CUpyG3C2tZ8kuMuuqc7gs

# This will:
# - Create directory /home/user/ripVID-worktrees/claude-audit
# - Check out branch claude/full-codebase-audit-011CUpyG3C2tZ8kuMuuqc7gs there
# - Link it to the main repository
```

#### 5. Set Up Dependencies in New Worktree

```bash
# Navigate to new worktree
cd /home/user/ripVID-worktrees/claude-audit

# Install dependencies (they're not shared between worktrees)
npm install

# Verify everything works
npm run dev
```

---

## Daily Workflow

### Starting a New Feature

```bash
# From main repository
cd /home/user/ripVID

# Create new branch and worktree in one command
git worktree add -b feature-new-downloader /home/user/ripVID-worktrees/feature-new-downloader

# Set up the new worktree
cd /home/user/ripVID-worktrees/feature-new-downloader
npm install

# Start development
npm run tauri:dev
```

### Working on Existing Worktree

```bash
# Navigate to worktree
cd /home/user/ripVID-worktrees/claude-audit

# Pull latest changes
git pull

# Make your changes
# ... edit files ...

# Commit as normal
git add .
git commit -m "feat: implement new feature"
git push
```

### Switching Between Worktrees

```bash
# Work on audit feature
cd /home/user/ripVID-worktrees/claude-audit
npm run tauri:dev

# Switch to another feature (in a new terminal)
cd /home/user/ripVID-worktrees/feature-ui-redesign
npm run tauri:dev

# Both can run simultaneously!
```

### Checking Status Across All Worktrees

```bash
# List all worktrees
git worktree list

# Output:
# /home/user/ripVID              7b61507 [main]
# /home/user/ripVID-worktrees/claude-audit  7b61507 [claude/full-codebase-audit-011CUpyG3C2tZ8kuMuuqc7gs]
```

---

## Testing in Worktrees

### Development Testing

```bash
# In your feature worktree
cd /home/user/ripVID-worktrees/claude-audit

# Run development server
npm run tauri:dev

# In another terminal, run tests
npm test  # (if you have tests)
```

### Build Testing

```bash
# Build the application
npm run build
npm run tauri:build

# Test the built application
# Installers will be in: src-tauri/target/release/bundle/
```

### Comparison Testing

```bash
# Terminal 1: Run stable version
cd /home/user/ripVID
npm run tauri:dev  # Runs on default port

# Terminal 2: Run feature version
cd /home/user/ripVID-worktrees/claude-audit
npm run tauri:dev  # May need to configure different port

# Compare behavior side-by-side
```

### Pre-Merge Testing Checklist

Before merging to main, test in the worktree:

```bash
cd /home/user/ripVID-worktrees/claude-audit

# 1. Clean build
rm -rf node_modules dist src-tauri/target
npm install
npm run build

# 2. Run development version
npm run tauri:dev
# Test all features manually

# 3. Build production version
npm run tauri:build
# Test the installer

# 4. Verify no uncommitted changes
git status

# 5. Ensure branch is up to date
git pull
```

---

## Best Practices

### Naming Conventions

Use descriptive, lowercase names with hyphens:

```bash
# Good names
git worktree add /home/user/ripVID-worktrees/feature-youtube-playlists
git worktree add /home/user/ripVID-worktrees/bugfix-download-timeout
git worktree add /home/user/ripVID-worktrees/refactor-ui-components
git worktree add /home/user/ripVID-worktrees/hotfix-security-issue

# Avoid
git worktree add /home/user/ripVID-worktrees/temp        # Too vague
git worktree add /home/user/ripVID-worktrees/test123     # Unclear purpose
git worktree add /home/user/ripVID-worktrees/FeatureNew  # Use lowercase
```

### When to Create New vs Use Existing

**Create New Worktree:**
- Starting a new feature or bugfix
- Need to preserve current build state while working on something else
- Reviewing a pull request
- Testing a hot fix while continuing feature work

**Use Existing Worktree:**
- Continuing work on an existing feature
- Making related changes to the same feature
- Iterating on the same branch

### Cleaning Up Old Worktrees

```bash
# List all worktrees
git worktree list

# Remove a worktree (after merging or abandoning)
git worktree remove /home/user/ripVID-worktrees/feature-old

# Or if there are uncommitted changes (force remove)
git worktree remove -f /home/user/ripVID-worktrees/feature-old

# Delete the branch if no longer needed
git branch -d feature-old

# Clean up stale worktree metadata
git worktree prune
```

**Regular Cleanup Schedule:**
- Weekly: Review active worktrees, remove merged branches
- After PR merge: Remove the worktree immediately
- Monthly: Run `git worktree prune` to clean up stale references

### Syncing Changes Between Worktrees

Worktrees share the same Git history, so changes are automatically visible:

```bash
# In worktree A: Make a commit
cd /home/user/ripVID-worktrees/feature-a
git add .
git commit -m "Add new feature"

# In worktree B: See the commit immediately
cd /home/user/ripVID-worktrees/feature-b
git log  # The commit from feature-a is visible

# To merge changes from main into your feature branch
git fetch origin
git merge origin/main
# Or use rebase
git rebase origin/main
```

### Branch Protection

**Protect Your Main Branch:**

```bash
# Keep main branch in the main directory
cd /home/user/ripVID
git checkout main

# Do ALL development in worktrees
# NEVER commit directly to main from the main directory

# Set up a pre-commit hook to warn if on main
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
branch=$(git symbolic-ref --short HEAD)
if [ "$branch" = "main" ]; then
    echo "ERROR: Direct commits to main are not allowed!"
    echo "Please use a feature branch in a worktree."
    exit 1
fi
EOF

chmod +x .git/hooks/pre-commit
```

---

## Common Commands Reference

### Creating Worktrees

```bash
# Create worktree for existing branch
git worktree add <path> <branch-name>
git worktree add /home/user/ripVID-worktrees/my-feature my-feature-branch

# Create worktree and new branch
git worktree add -b <new-branch> <path>
git worktree add -b feature-new /home/user/ripVID-worktrees/feature-new

# Create worktree from specific commit
git worktree add <path> <commit-hash>
```

### Managing Worktrees

```bash
# List all worktrees
git worktree list

# Detailed list
git worktree list --porcelain

# Remove a worktree
git worktree remove <path>
git worktree remove /home/user/ripVID-worktrees/old-feature

# Force remove (with uncommitted changes)
git worktree remove -f <path>

# Remove stale worktree metadata
git worktree prune

# Move a worktree to new location
git worktree move <old-path> <new-path>

# Lock a worktree (prevent removal)
git worktree lock <path>

# Unlock a worktree
git worktree unlock <path>
```

### Working in Worktrees

```bash
# All regular git commands work the same
cd /home/user/ripVID-worktrees/my-feature

git status
git add .
git commit -m "message"
git push
git pull
git merge
git rebase

# Switch branches within a worktree (not recommended, defeats the purpose)
git checkout other-branch  # This works but you lose worktree benefits
```

### Navigation Shortcuts

```bash
# Add to your ~/.bashrc or ~/.zshrc

# Quick navigation
alias ripvid='cd /home/user/ripVID'
alias ripwork='cd /home/user/ripVID-worktrees'

# List worktrees
alias gwl='git worktree list'

# Remove worktree
alias gwr='git worktree remove'

# Function to create and set up new worktree
ripnew() {
    local branch_name=$1
    local worktree_name=${2:-$branch_name}
    cd /home/user/ripVID
    git worktree add -b "$branch_name" "/home/user/ripVID-worktrees/$worktree_name"
    cd "/home/user/ripVID-worktrees/$worktree_name"
    npm install
    echo "Worktree created and dependencies installed!"
}

# Usage: ripnew feature-awesome-downloader
```

---

## Common Pitfalls & Troubleshooting

### Pitfall 1: Forgetting to Install Dependencies

**Problem:**
```bash
git worktree add /home/user/ripVID-worktrees/feature-new
cd /home/user/ripVID-worktrees/feature-new
npm run dev  # ERROR: Cannot find module...
```

**Solution:**
```bash
# Always install dependencies in new worktrees
npm install

# Or copy node_modules (faster but risky if versions differ)
cp -r /home/user/ripVID/node_modules /home/user/ripVID-worktrees/feature-new/
```

### Pitfall 2: Cannot Check Out Branch in Multiple Worktrees

**Problem:**
```bash
git worktree add /home/user/ripVID-worktrees/test main
# ERROR: 'main' is already checked out at '/home/user/ripVID'
```

**Why:** Git prevents checking out the same branch in multiple worktrees to avoid conflicts.

**Solution:**
```bash
# Create a new branch based on main
git worktree add -b test-main /home/user/ripVID-worktrees/test main

# Or work directly in the main directory for main branch changes
cd /home/user/ripVID
git checkout main
```

### Pitfall 3: Stale Worktrees After Manual Deletion

**Problem:**
```bash
# Manually deleted directory
rm -rf /home/user/ripVID-worktrees/old-feature

# Git still thinks it exists
git worktree list
# Shows: /home/user/ripVID-worktrees/old-feature [???]
```

**Solution:**
```bash
# Clean up stale references
git worktree prune

# If branch is locked
git worktree unlock /home/user/ripVID-worktrees/old-feature
git worktree prune
```

### Pitfall 4: Large Disk Usage

**Problem:**
Each worktree has its own build artifacts and dependencies, multiplying disk usage.

**For ripVID:**
- `node_modules/`: ~500MB per worktree
- `src-tauri/target/`: ~2-5GB per worktree (Rust builds are large!)
- `dist/`: ~10-50MB per worktree

**Solution:**
```bash
# Regularly clean up unused worktrees
git worktree list
git worktree remove /home/user/ripVID-worktrees/old-feature

# Clean build artifacts in inactive worktrees
cd /home/user/ripVID-worktrees/inactive-feature
rm -rf src-tauri/target dist

# Rebuild when needed
npm run build
```

### Pitfall 5: Confusion About Which Worktree You're In

**Problem:**
Working in the wrong worktree and making commits to the wrong branch.

**Solution:**
```bash
# Add to your shell prompt to show current branch and worktree
# For bash, add to ~/.bashrc:
parse_git_branch() {
    git branch 2> /dev/null | sed -e '/^[^*]/d' -e 's/* \(.*\)/(\1)/'
}
export PS1="\w \$(parse_git_branch) $ "

# Or use a tool like 'starship' for a better prompt

# Always check before committing
git status
pwd
```

### Pitfall 6: Port Conflicts When Running Multiple Dev Servers

**Problem:**
```bash
# Terminal 1
cd /home/user/ripVID
npm run tauri:dev  # Uses default port

# Terminal 2
cd /home/user/ripVID-worktrees/feature-new
npm run tauri:dev  # ERROR: Port already in use
```

**Solution:**
```bash
# Configure different ports in vite.config.ts for each worktree
# Or manually specify port
npm run dev -- --port 5174

# For Tauri, both frontend and backend need different ports
# Edit vite.config.ts in the worktree:
export default defineConfig({
  server: {
    port: 5174  // Change this for each worktree
  }
})
```

### Pitfall 7: Forgetting to Push Branches

**Problem:**
Worktrees are local. If you create a new branch in a worktree, it won't be on the remote until you push.

**Solution:**
```bash
cd /home/user/ripVID-worktrees/new-feature

# After first commit
git push -u origin new-feature

# Subsequent pushes
git push
```

### Troubleshooting Commands

```bash
# Check worktree status
git worktree list
git worktree list --porcelain

# Check which branch is checked out where
git branch -v

# Verify repository integrity
git fsck

# Check worktree location from any worktree
git rev-parse --show-toplevel     # Current worktree root
git rev-parse --git-common-dir    # Shared .git directory

# Repair a broken worktree link
cd /home/user/ripVID-worktrees/feature-name
git worktree repair

# If everything is broken, re-clone the repository
cd /home/user
git clone <repository-url> ripVID-fresh
cd ripVID-fresh
git worktree add /home/user/ripVID-worktrees/feature-name feature-branch
```

---

## Integration with ripVID Workflow

### Current State

Your current setup:
- **Main directory:** `/home/user/ripVID`
- **Current branch:** `claude/full-codebase-audit-011CUpyG3C2tZ8kuMuuqc7gs`
- **No main branch yet** (or not checked out locally)

### Recommended Workflow

#### Phase 1: Set Up Main Branch (Stable Foundation)

```bash
cd /home/user/ripVID

# Option A: If you have a main branch remotely
git fetch origin
git checkout -b main origin/main

# Option B: If you don't have a main branch, create from current stable state
# (Do this when your audit branch is merged or from the last stable commit)
git checkout -b main
git push -u origin main

# Stay on your audit branch for now
git checkout claude/full-codebase-audit-011CUpyG3C2tZ8kuMuuqc7gs
```

#### Phase 2: Move Current Work to Worktree

```bash
cd /home/user/ripVID

# Create worktree for your audit branch
git worktree add /home/user/ripVID-worktrees/claude-audit claude/full-codebase-audit-011CUpyG3C2tZ8kuMuuqc7gs

# Set up the worktree
cd /home/user/ripVID-worktrees/claude-audit
npm install

# Verify it works
npm run tauri:dev

# Once verified, switch main directory to main branch
cd /home/user/ripVID
git checkout main

# Now you have:
# - /home/user/ripVID → main (stable)
# - /home/user/ripVID-worktrees/claude-audit → audit branch (development)
```

#### Phase 3: Safe Development Workflow

```bash
# All development happens in worktrees
cd /home/user/ripVID-worktrees/claude-audit

# Make changes
# ... edit files ...

# Commit and push
git add .
git commit -m "feat: implement security improvements"
git push

# Test thoroughly
npm run tauri:dev  # Manual testing
npm run tauri:build  # Build testing

# When ready to merge:
# Option A: Via Pull Request (Recommended)
# - Create PR on GitHub/GitLab
# - Review and merge through web interface

# Option B: Local merge (for small changes)
cd /home/user/ripVID
git checkout main
git pull origin main  # Ensure up-to-date
git merge --no-ff claude/full-codebase-audit-011CUpyG3C2tZ8kuMuuqc7gs
git push origin main

# Clean up worktree after merge
git worktree remove /home/user/ripVID-worktrees/claude-audit
git branch -d claude/full-codebase-audit-011CUpyG3C2tZ8kuMuuqc7gs
```

#### Phase 4: Ongoing Development

```bash
# Main directory always stays on main branch
cd /home/user/ripVID
git checkout main  # Should always be on main

# For each new feature/fix:
git worktree add -b feature-new-downloader /home/user/ripVID-worktrees/feature-new-downloader
cd /home/user/ripVID-worktrees/feature-new-downloader
npm install
# ... develop ...
# ... test ...
# ... merge via PR ...
git worktree remove /home/user/ripVID-worktrees/feature-new-downloader
```

### Testing Strategy

#### 1. Development Testing (In Worktree)
```bash
cd /home/user/ripVID-worktrees/claude-audit

# Frontend dev server
npm run dev

# Full Tauri app
npm run tauri:dev

# Build and test
npm run build
npm run tauri:build
```

#### 2. Comparison Testing (Stable vs Feature)
```bash
# Terminal 1: Stable version
cd /home/user/ripVID
npm run tauri:dev

# Terminal 2: Feature version
cd /home/user/ripVID-worktrees/claude-audit
npm run tauri:dev

# Compare behavior, performance, UI
```

#### 3. Pre-Merge Testing (In Main Directory)
```bash
# Test merge before actually merging
cd /home/user/ripVID
git checkout main

# Test merge (no-commit flag)
git merge --no-commit --no-ff claude/full-codebase-audit-011CUpyG3C2tZ8kuMuuqc7gs

# Build and test
npm install  # In case dependencies changed
npm run build
npm run tauri:build

# If tests fail
git merge --abort  # Abort the merge
# Go back to worktree and fix issues

# If tests pass
git merge --abort  # Still abort
# Now merge for real (via PR or commit the merge)
```

### Branch Protection Strategy

```bash
# In main directory, set up pre-commit hook
cd /home/user/ripVID

# Create hook to prevent direct commits to main
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash

branch=$(git symbolic-ref --short HEAD)

if [ "$branch" = "main" ]; then
    echo "=========================================="
    echo "ERROR: Direct commits to main are blocked!"
    echo "=========================================="
    echo ""
    echo "Please use a worktree for development:"
    echo "  1. Create worktree: git worktree add -b feature-name /home/user/ripVID-worktrees/feature-name"
    echo "  2. Develop in worktree: cd /home/user/ripVID-worktrees/feature-name"
    echo "  3. Merge via PR when ready"
    echo ""
    exit 1
fi

# If you're in a worktree, allow commits
worktree=$(git rev-parse --show-toplevel)
if [ "$worktree" != "/home/user/ripVID" ]; then
    # In a worktree, allow commits
    exit 0
fi

# Shouldn't reach here, but block just in case
exit 1
EOF

chmod +x .git/hooks/pre-commit

# Test it
git checkout main
echo "test" >> README.md
git add README.md
git commit -m "test"  # Should fail with error message

git checkout claude/full-codebase-audit-011CUpyG3C2tZ8kuMuuqc7gs
git restore README.md
```

### Release Workflow

```bash
# 1. All features are developed in worktrees
# 2. Each feature is thoroughly tested in its worktree
# 3. Features are merged to main via PR
# 4. For releases, use the main directory

cd /home/user/ripVID
git checkout main
git pull origin main

# Ensure clean state
git status

# Build production release
npm install
npm run build
npm run tauri:build

# Tag release
git tag -a v2.2.0 -m "Release version 2.2.0"
git push origin v2.2.0

# Installers are in: src-tauri/target/release/bundle/
```

### Hotfix Workflow

```bash
# Urgent bug found in production!

# Create hotfix worktree from main
cd /home/user/ripVID
git worktree add -b hotfix-critical-bug /home/user/ripVID-worktrees/hotfix-critical-bug main

# Fix the bug
cd /home/user/ripVID-worktrees/hotfix-critical-bug
npm install

# Make fix
# ... edit files ...

# Test thoroughly
npm run tauri:dev
npm run tauri:build

# Commit and push
git add .
git commit -m "fix: critical security vulnerability"
git push -u origin hotfix-critical-bug

# Create PR or merge directly
cd /home/user/ripVID
git checkout main
git merge --no-ff hotfix-critical-bug
git push origin main

# Tag hotfix release
git tag -a v2.1.2 -m "Hotfix: critical security fix"
git push origin v2.1.2

# Clean up
git worktree remove /home/user/ripVID-worktrees/hotfix-critical-bug
git branch -d hotfix-critical-bug
```

---

## Quick Start Checklist

Use this checklist to get started:

- [ ] **Step 1:** Ensure you have a main branch
  ```bash
  cd /home/user/ripVID
  git branch -a | grep main
  ```

- [ ] **Step 2:** Create worktrees directory
  ```bash
  mkdir -p /home/user/ripVID-worktrees
  ```

- [ ] **Step 3:** Create worktree for current work
  ```bash
  git worktree add /home/user/ripVID-worktrees/claude-audit claude/full-codebase-audit-011CUpyG3C2tZ8kuMuuqc7gs
  ```

- [ ] **Step 4:** Set up worktree
  ```bash
  cd /home/user/ripVID-worktrees/claude-audit
  npm install
  ```

- [ ] **Step 5:** Verify it works
  ```bash
  npm run tauri:dev
  ```

- [ ] **Step 6:** Switch main directory to main branch
  ```bash
  cd /home/user/ripVID
  git checkout main
  ```

- [ ] **Step 7:** Set up branch protection hook (optional)
  ```bash
  # See "Branch Protection Strategy" section above
  ```

- [ ] **Step 8:** Add shell aliases (optional)
  ```bash
  # Add to ~/.bashrc or ~/.zshrc
  # See "Navigation Shortcuts" section above
  ```

---

## Summary

Git worktrees enable:
- **Safe parallel development:** Main stays stable, features in worktrees
- **Fast context switching:** No rebuild needed
- **Thorough testing:** Test before merging to main
- **Risk mitigation:** Experimental work is isolated

**Golden Rules:**
1. **Keep main in the main directory** - Always stable and deployable
2. **All development in worktrees** - One worktree per feature/fix
3. **Test thoroughly in worktrees** - Never merge untested code
4. **Clean up after merging** - Remove worktrees for merged branches
5. **Never push directly to main** - Use PRs or careful merging

---

## Additional Resources

- [Git Worktree Documentation](https://git-scm.com/docs/git-worktree)
- [Git Worktree Tutorial](https://git-scm.com/docs/git-worktree#_synopsis)
- [Tauri Build Guide](https://tauri.app/v1/guides/building/)

---

**Version:** 1.0
**Last Updated:** 2025-11-05
**Project:** ripVID v2.1.0
