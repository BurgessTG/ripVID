#!/bin/bash

# Git Worktree Setup Script for ripVID
# This script helps set up a safe worktree-based development workflow

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_DIR="/home/user/ripVID"
WORKTREES_DIR="/home/user/ripVID-worktrees"
CURRENT_BRANCH="claude/full-codebase-audit-011CUpyG3C2tZ8kuMuuqc7gs"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  ripVID Worktree Setup Script${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Function to print status messages
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_info() {
    echo -e "${BLUE}[i]${NC} $1"
}

# Check if we're in the right directory
if [ ! -d "$REPO_DIR/.git" ]; then
    print_error "Not in a git repository at $REPO_DIR"
    exit 1
fi

cd "$REPO_DIR"

print_info "Current directory: $(pwd)"
echo ""

# Step 1: Check current branch
print_info "Step 1: Checking current branch..."
CURRENT=$(git branch --show-current)
print_info "Currently on branch: $CURRENT"
echo ""

# Step 2: Check if main branch exists
print_info "Step 2: Checking for main branch..."
if git show-ref --verify --quiet refs/heads/main; then
    print_status "Main branch exists locally"
    HAS_MAIN=true
elif git show-ref --verify --quiet refs/remotes/origin/main; then
    print_warning "Main branch exists on remote but not locally"
    print_info "Creating local main branch..."
    git checkout -b main origin/main
    git checkout "$CURRENT"
    print_status "Local main branch created"
    HAS_MAIN=true
else
    print_warning "No main branch found"
    print_info "You'll need to create a main branch later"
    print_info "You can run: git checkout -b main && git push -u origin main"
    HAS_MAIN=false
fi
echo ""

# Step 3: Create worktrees directory
print_info "Step 3: Creating worktrees directory..."
if [ -d "$WORKTREES_DIR" ]; then
    print_warning "Worktrees directory already exists: $WORKTREES_DIR"
else
    mkdir -p "$WORKTREES_DIR"
    print_status "Created worktrees directory: $WORKTREES_DIR"
fi
echo ""

# Step 4: List existing worktrees
print_info "Step 4: Checking existing worktrees..."
if git worktree list | grep -q "$WORKTREES_DIR"; then
    print_info "Existing worktrees found:"
    git worktree list
else
    print_info "No worktrees found yet"
fi
echo ""

# Step 5: Offer to create worktree for current branch
print_info "Step 5: Setting up worktree for current work..."
WORKTREE_NAME="claude-audit"
WORKTREE_PATH="$WORKTREES_DIR/$WORKTREE_NAME"

if [ -d "$WORKTREE_PATH" ]; then
    print_warning "Worktree already exists at: $WORKTREE_PATH"
    read -p "Do you want to remove and recreate it? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "Removing existing worktree..."
        git worktree remove -f "$WORKTREE_PATH" || true
        rm -rf "$WORKTREE_PATH"
        print_status "Removed old worktree"
    else
        print_info "Skipping worktree creation"
        SKIP_WORKTREE=true
    fi
fi

if [ "$SKIP_WORKTREE" != "true" ]; then
    print_info "Creating worktree for branch: $CURRENT_BRANCH"
    git worktree add "$WORKTREE_PATH" "$CURRENT_BRANCH"
    print_status "Worktree created at: $WORKTREE_PATH"

    # Install dependencies
    print_info "Installing dependencies in worktree..."
    cd "$WORKTREE_PATH"
    npm install
    print_status "Dependencies installed"
    cd "$REPO_DIR"
fi
echo ""

# Step 6: Set up branch protection hook
print_info "Step 6: Setting up branch protection..."
HOOK_FILE="$REPO_DIR/.git/hooks/pre-commit"

if [ -f "$HOOK_FILE" ]; then
    print_warning "Pre-commit hook already exists"
    read -p "Do you want to replace it with branch protection hook? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Keeping existing hook"
        SKIP_HOOK=true
    fi
fi

if [ "$SKIP_HOOK" != "true" ]; then
    cat > "$HOOK_FILE" << 'EOF'
#!/bin/bash

# Branch protection hook for ripVID
# Prevents direct commits to main branch in main directory

branch=$(git symbolic-ref --short HEAD)
worktree=$(git rev-parse --show-toplevel)

if [ "$branch" = "main" ] && [ "$worktree" = "/home/user/ripVID" ]; then
    echo "=========================================="
    echo "ERROR: Direct commits to main are blocked!"
    echo "=========================================="
    echo ""
    echo "Please use a worktree for development:"
    echo "  1. Create worktree:"
    echo "     git worktree add -b feature-name /home/user/ripVID-worktrees/feature-name"
    echo ""
    echo "  2. Develop in worktree:"
    echo "     cd /home/user/ripVID-worktrees/feature-name"
    echo ""
    echo "  3. Merge via PR when ready"
    echo ""
    exit 1
fi

# Allow commits in worktrees or on other branches
exit 0
EOF

    chmod +x "$HOOK_FILE"
    print_status "Branch protection hook installed"
fi
echo ""

# Step 7: Suggest shell aliases
print_info "Step 7: Optional shell aliases..."
echo ""
echo "Add these aliases to your ~/.bashrc or ~/.zshrc for easier worktree management:"
echo ""
cat << 'EOF'
# ripVID Worktree Aliases
alias ripvid='cd /home/user/ripVID'
alias ripwork='cd /home/user/ripVID-worktrees'
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
EOF
echo ""

# Step 8: Show summary
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}  Setup Complete!${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

print_info "Summary:"
echo "  Repository: $REPO_DIR"
echo "  Worktrees:  $WORKTREES_DIR"
echo ""

if [ "$HAS_MAIN" = true ]; then
    print_status "Main branch is set up"
else
    print_warning "Main branch needs to be created"
    echo "  Run: git checkout -b main && git push -u origin main"
fi

if [ "$SKIP_WORKTREE" != "true" ]; then
    print_status "Worktree created for current work"
    echo "  Path: $WORKTREE_PATH"
    echo "  Branch: $CURRENT_BRANCH"
else
    print_info "Worktree was skipped or already exists"
fi

if [ "$SKIP_HOOK" != "true" ]; then
    print_status "Branch protection hook installed"
else
    print_info "Branch protection hook was skipped"
fi

echo ""
print_info "Next steps:"
echo ""
echo "  1. Start working in your worktree:"
echo "     cd $WORKTREE_PATH"
echo "     npm run tauri:dev"
echo ""
echo "  2. When ready, switch main directory to main branch:"
echo "     cd $REPO_DIR"
if [ "$HAS_MAIN" = true ]; then
    echo "     git checkout main"
else
    echo "     git checkout -b main  # Create main branch first"
fi
echo ""
echo "  3. Create new worktrees for new features:"
echo "     git worktree add -b feature-name $WORKTREES_DIR/feature-name"
echo ""
echo "  4. Read the full guide:"
echo "     cat GIT_WORKTREES_GUIDE.md"
echo "     cat WORKTREE_QUICK_REFERENCE.md"
echo ""

# List current worktrees
echo -e "${BLUE}Current worktrees:${NC}"
git worktree list
echo ""

print_status "All done! Happy coding!"
