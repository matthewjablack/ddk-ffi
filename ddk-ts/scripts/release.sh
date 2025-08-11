#!/usr/bin/env bash
set -euo pipefail
cd {{justfile_directory()}}/ddk-ts

# Check if working directory is clean
if [[ -n $(git status --porcelain) ]]; then
    echo "Error: Working directory is not clean. Please commit or stash changes."
    exit 1
fi

# Update version
npm version {{version}} --no-git-tag-version

# Build for all platforms
echo "Building for all platforms..."
# pnpm build:darwin-arm64
pnpm build

# Run tests
echo "Running tests..."
pnpm test

# Commit changes
git add -A
git commit -m "chore(ddk-ts): release v{{version}}"

# Create tag
git tag "v{{version}}"

# Push to GitHub
git push -u origin node-bindings
git push origin "v{{version}}"

# Publish to npm
echo "Publishing to npm..."
npm publish --access public

echo "✅ Released ddk-ts v{{version}} successfully!"