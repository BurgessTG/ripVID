#!/bin/bash

# Worktree Helper Script for ripVID
# Provides easy commands for common worktree operations

REPO_DIR="/home/user/ripVID"
WORKTREES_DIR="/home/user/ripVID-worktrees"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_info() {
    echo -e "${BLUE}[i]${NC} $1"
}

print_usage() {
    echo "Usage: $0 <command> [arguments]"
    echo ""
    echo "Commands:"
    echo "  new <branch-name>         Create new worktree with new branch"
    echo "  add <branch-name>         Create worktree for existing branch"
    echo "  list                      List all worktrees"
    echo "  remove <worktree-name>    Remove a worktree"
    echo "  clean                     Remove all merged worktrees"
    echo "  prune                     Clean up stale worktree references"
    echo "  goto <worktree-name>      Print cd command to go to worktree"
    echo "  status                    Show status of all worktrees"
    echo ""
    echo "Examples:"
    echo "  $0 new feature-awesome"
    echo "  $0 add existing-branch"
    echo "  $0 list"
    echo "  $0 remove feature-old"
    echo "  $0 goto claude-audit"
    echo ""
}

cmd_new() {
    if [ -z "$1" ]; then
        print_error "Branch name required"
        echo "Usage: $0 new <branch-name>"
        exit 1
    fi

    local branch_name=$1
    local worktree_name=${2:-$branch_name}
    local worktree_path="$WORKTREES_DIR/$worktree_name"

    if [ -d "$worktree_path" ]; then
        print_error "Worktree already exists: $worktree_path"
        exit 1
    fi

    print_info "Creating new branch and worktree: $branch_name"
    cd "$REPO_DIR"
    git worktree add -b "$branch_name" "$worktree_path"

    print_info "Installing dependencies..."
    cd "$worktree_path"
    npm install

    print_status "Worktree created and ready!"
    echo ""
    echo "To start working:"
    echo "  cd $worktree_path"
    echo "  npm run tauri:dev"
}

cmd_add() {
    if [ -z "$1" ]; then
        print_error "Branch name required"
        echo "Usage: $0 add <existing-branch-name>"
        exit 1
    fi

    local branch_name=$1
    local worktree_name=${2:-$branch_name}
    local worktree_path="$WORKTREES_DIR/$worktree_name"

    if [ -d "$worktree_path" ]; then
        print_error "Worktree already exists: $worktree_path"
        exit 1
    fi

    # Check if branch exists
    cd "$REPO_DIR"
    if ! git show-ref --verify --quiet "refs/heads/$branch_name"; then
        print_error "Branch does not exist: $branch_name"
        echo "Use '$0 new $branch_name' to create a new branch"
        exit 1
    fi

    print_info "Creating worktree for existing branch: $branch_name"
    git worktree add "$worktree_path" "$branch_name"

    print_info "Installing dependencies..."
    cd "$worktree_path"
    npm install

    print_status "Worktree created and ready!"
    echo ""
    echo "To start working:"
    echo "  cd $worktree_path"
    echo "  npm run tauri:dev"
}

cmd_list() {
    cd "$REPO_DIR"
    print_info "All worktrees:"
    echo ""
    git worktree list
}

cmd_remove() {
    if [ -z "$1" ]; then
        print_error "Worktree name required"
        echo "Usage: $0 remove <worktree-name>"
        exit 1
    fi

    local worktree_name=$1
    local worktree_path="$WORKTREES_DIR/$worktree_name"

    if [ ! -d "$worktree_path" ]; then
        print_error "Worktree does not exist: $worktree_path"
        exit 1
    fi

    cd "$REPO_DIR"

    # Check for uncommitted changes
    cd "$worktree_path"
    if ! git diff-index --quiet HEAD --; then
        echo ""
        print_error "Worktree has uncommitted changes!"
        echo ""
        git status --short
        echo ""
        read -p "Force remove anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "Cancelled"
            exit 0
        fi
        FORCE="-f"
    fi

    cd "$REPO_DIR"
    print_info "Removing worktree: $worktree_name"
    git worktree remove $FORCE "$worktree_path"

    print_status "Worktree removed"

    # Ask about branch
    local branch=$(basename "$worktree_name")
    if git show-ref --verify --quiet "refs/heads/$branch"; then
        read -p "Delete branch '$branch' too? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            git branch -d "$branch" 2>/dev/null || git branch -D "$branch"
            print_status "Branch deleted"
        fi
    fi
}

cmd_clean() {
    cd "$REPO_DIR"
    print_info "Finding merged branches in worktrees..."
    echo ""

    local removed=0

    # Get list of merged branches
    git branch --merged main | grep -v "^\*" | grep -v "main" | while read -r branch; do
        branch=$(echo "$branch" | xargs)  # Trim whitespace

        # Check if there's a worktree for this branch
        local worktree_path=$(git worktree list --porcelain | grep -A 2 "branch refs/heads/$branch" | grep "worktree" | awk '{print $2}')

        if [ -n "$worktree_path" ] && [[ "$worktree_path" == "$WORKTREES_DIR"* ]]; then
            print_info "Found merged worktree: $worktree_path"
            git worktree remove "$worktree_path"
            git branch -d "$branch"
            print_status "Removed worktree and branch: $branch"
            removed=$((removed + 1))
        fi
    done

    if [ $removed -eq 0 ]; then
        print_info "No merged worktrees to clean up"
    else
        print_status "Cleaned up $removed worktree(s)"
    fi
}

cmd_prune() {
    cd "$REPO_DIR"
    print_info "Pruning stale worktree references..."
    git worktree prune
    print_status "Done"
}

cmd_goto() {
    if [ -z "$1" ]; then
        print_error "Worktree name required"
        echo "Usage: $0 goto <worktree-name>"
        exit 1
    fi

    local worktree_name=$1
    local worktree_path="$WORKTREES_DIR/$worktree_name"

    if [ ! -d "$worktree_path" ]; then
        print_error "Worktree does not exist: $worktree_path"
        exit 1
    fi

    echo "cd $worktree_path"
}

cmd_status() {
    cd "$REPO_DIR"
    print_info "Worktree Status Report"
    echo ""

    git worktree list --porcelain | while IFS= read -r line; do
        if [[ $line == worktree* ]]; then
            worktree_path=${line#worktree }
            echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
            echo -e "${GREEN}Worktree:${NC} $worktree_path"
        elif [[ $line == branch* ]]; then
            branch=${line#branch }
            branch=${branch#refs/heads/}
            echo -e "${GREEN}Branch:${NC} $branch"
        elif [[ $line == HEAD* ]]; then
            head=${line#HEAD }
            echo -e "${GREEN}Commit:${NC} ${head:0:7}"

            # Get status for this worktree
            if [ -d "$worktree_path" ]; then
                cd "$worktree_path"

                # Check for uncommitted changes
                if ! git diff-index --quiet HEAD -- 2>/dev/null; then
                    echo -e "${YELLOW}Status:${NC} Has uncommitted changes"
                else
                    echo -e "${GREEN}Status:${NC} Clean"
                fi

                # Check for unpushed commits
                local unpushed=$(git log @{u}.. --oneline 2>/dev/null | wc -l)
                if [ "$unpushed" -gt 0 ]; then
                    echo -e "${YELLOW}Unpushed:${NC} $unpushed commit(s)"
                fi

                # Check disk usage
                if [ -d "node_modules" ]; then
                    local size=$(du -sh . 2>/dev/null | cut -f1)
                    echo -e "${BLUE}Size:${NC} $size"
                fi
            fi
        fi
    done

    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# Main command dispatcher
case "$1" in
    new)
        cmd_new "$2" "$3"
        ;;
    add)
        cmd_add "$2" "$3"
        ;;
    list)
        cmd_list
        ;;
    remove)
        cmd_remove "$2"
        ;;
    clean)
        cmd_clean
        ;;
    prune)
        cmd_prune
        ;;
    goto)
        cmd_goto "$2"
        ;;
    status)
        cmd_status
        ;;
    *)
        print_usage
        exit 1
        ;;
esac
