# Git Worktree Quick Reference Card

**Essential commands for daily worktree usage with ripVID**

---

## Directory Structure

```
/home/user/ripVID/              # Main worktree (always on main branch)
/home/user/ripVID-worktrees/    # All development worktrees
  ├── claude-audit/
  ├── feature-xyz/
  └── hotfix-abc/
```

---

## Common Commands

### Create Worktrees

```bash
# Create worktree for existing branch
git worktree add /home/user/ripVID-worktrees/<name> <existing-branch>

# Create worktree with new branch
git worktree add -b <new-branch> /home/user/ripVID-worktrees/<name>

# Create worktree from main
git worktree add -b <new-branch> /home/user/ripVID-worktrees/<name> main
```

### List and Manage

```bash
# List all worktrees
git worktree list

# Remove worktree
git worktree remove /home/user/ripVID-worktrees/<name>

# Force remove (with uncommitted changes)
git worktree remove -f /home/user/ripVID-worktrees/<name>

# Clean up stale references
git worktree prune
```

### After Creating Worktree

```bash
cd /home/user/ripVID-worktrees/<name>
npm install              # Install dependencies
npm run tauri:dev        # Start development server
```

---

## Daily Workflow

### Start New Feature

```bash
cd /home/user/ripVID
git worktree add -b feature-name /home/user/ripVID-worktrees/feature-name
cd /home/user/ripVID-worktrees/feature-name
npm install
# Start coding...
```

### Work on Feature

```bash
cd /home/user/ripVID-worktrees/feature-name
# Make changes...
git add .
git commit -m "feat: description"
git push
```

### Test Before Merge

```bash
cd /home/user/ripVID-worktrees/feature-name
npm run build
npm run tauri:build
# Test thoroughly...
```

### Merge to Main

```bash
# Via Pull Request (Recommended)
# - Create PR on GitHub
# - Review and merge

# Or locally
cd /home/user/ripVID
git checkout main
git pull
git merge --no-ff feature-name
git push
```

### Clean Up After Merge

```bash
cd /home/user/ripVID
git worktree remove /home/user/ripVID-worktrees/feature-name
git branch -d feature-name
```

---

## Useful Aliases

Add to `~/.bashrc` or `~/.zshrc`:

```bash
# Navigation
alias ripvid='cd /home/user/ripVID'
alias ripwork='cd /home/user/ripVID-worktrees'

# Worktree management
alias gwl='git worktree list'
alias gwr='git worktree remove'
alias gwp='git worktree prune'

# Create new worktree with setup
ripnew() {
    local branch_name=$1
    local worktree_name=${2:-$branch_name}
    cd /home/user/ripVID
    git worktree add -b "$branch_name" "/home/user/ripVID-worktrees/$worktree_name"
    cd "/home/user/ripVID-worktrees/$worktree_name"
    npm install
    echo "Ready! Run 'npm run tauri:dev' to start."
}
```

Usage: `ripnew feature-awesome`

---

## Testing Commands

```bash
# Development
npm run dev              # Frontend only
npm run tauri:dev        # Full Tauri app

# Build
npm run build            # Build frontend
npm run tauri:build      # Build full app with installers

# Clean build
rm -rf node_modules dist src-tauri/target
npm install
npm run build
```

---

## Troubleshooting

### Port already in use

```bash
# Edit vite.config.ts in worktree
export default defineConfig({
  server: {
    port: 5174  // Change for each worktree
  }
})
```

### Branch already checked out

```bash
# Can't check out same branch twice
# Create new branch instead
git worktree add -b new-branch-name /home/user/ripVID-worktrees/<name> existing-branch
```

### Worktree manually deleted

```bash
git worktree prune
```

---

## Golden Rules

1. **Main directory = main branch only** (stable, no direct commits)
2. **All development in worktrees** (one per feature)
3. **Always `npm install` in new worktrees**
4. **Test thoroughly before merging**
5. **Clean up merged worktrees**

---

## Emergency: Something Broke

```bash
# Check status
git worktree list
git status

# Repair worktree
cd /home/user/ripVID-worktrees/<name>
git worktree repair

# Nuclear option: Remove and recreate
cd /home/user/ripVID
git worktree remove -f /home/user/ripVID-worktrees/<name>
git worktree prune
git worktree add /home/user/ripVID-worktrees/<name> <branch>
cd /home/user/ripVID-worktrees/<name>
npm install
```

---

**For full guide, see:** `GIT_WORKTREES_GUIDE.md`
