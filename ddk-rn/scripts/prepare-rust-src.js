#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const ddkRnRoot = path.join(__dirname, '..');
const projectRoot = path.join(ddkRnRoot, '..');
const ddkFfiSource = path.join(projectRoot, 'ddk-ffi');
const ddkFfiDest = path.join(ddkRnRoot, 'ddk-ffi');

console.log('📦 Preparing Rust source for npm package...');

function copyDirectory(src, dest) {
  // Remove existing destination if it exists
  if (fs.existsSync(dest)) {
    console.log(`🗑️  Removing existing ${path.basename(dest)}...`);
    execSync(`rm -rf "${dest}"`, { stdio: 'inherit' });
  }

  // Create destination directory
  fs.mkdirSync(dest, { recursive: true });

  // Copy all files except target directory
  console.log(`📋 Copying ${path.basename(src)} to ${path.basename(dest)}...`);
  execSync(`rsync -av --exclude=target/ --exclude=.git/ "${src}/" "${dest}/"`, { 
    stdio: 'inherit' 
  });
}

function main() {
  try {
    // Check if source exists
    if (!fs.existsSync(ddkFfiSource)) {
      console.error(`❌ Source directory not found: ${ddkFfiSource}`);
      process.exit(1);
    }

    // Copy the Rust source
    copyDirectory(ddkFfiSource, ddkFfiDest);

    console.log('✅ Rust source prepared for npm package');
    console.log(`📁 Location: ${ddkFfiDest}`);
    
  } catch (error) {
    console.error('❌ Failed to prepare Rust source:', error.message);
    process.exit(1);
  }
}

main();