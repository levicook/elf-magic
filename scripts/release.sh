#!/bin/bash
set -e

BUMP_TYPE=${1:-patch}

# Validate bump type
case $BUMP_TYPE in
patch | minor | major)
    echo "🚀 Rolling $BUMP_TYPE version with odo..."
    ;;
*)
    echo "❌ Invalid bump type: $BUMP_TYPE"
    echo "Usage: $0 [patch|minor|major]"
    exit 1
    ;;
esac

# Ensure we're in a clean git state
if [[ -n $(git status --porcelain) ]]; then
    echo "❌ Working directory is not clean. Please commit or stash changes first."
    git status --short
    exit 1
fi

# Show current version
echo "📊 Current version: $(odo show --format=plain 2>/dev/null || odo show)"

# Roll the version using our own tool! 🎯
odo roll $BUMP_TYPE

# Get the new version
VERSION=$(odo show | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' | head -1)
echo "📝 New version: $VERSION"

# Run CI to make sure everything still works
echo "🧪 Running quick CI checks..."
make ci

echo "💾 Committing and tagging..."
git add -A
git commit -m "Release v$VERSION"
git tag v$VERSION

echo "📤 Pushing to GitHub..."
git push origin main v$VERSION

echo "✅ Release v$VERSION triggered! Check GitHub Actions at:"
echo "   https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/actions"
