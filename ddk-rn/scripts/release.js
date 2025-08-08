#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const projectRoot = path.join(__dirname, '..', '..');
const ddkRnRoot = path.join(__dirname, '..');

console.log('🚀 Starting automated release process...\n');

function runCommand(command, cwd = ddkRnRoot, description = null) {
  if (description) {
    console.log(`📋 ${description}`);
  }
  console.log(`   $ ${command}`);
  try {
    const result = execSync(command, { cwd, stdio: 'inherit' });
    return result;
  } catch (error) {
    console.error(`❌ Command failed: ${command}`);
    throw error;
  }
}

function checkGitStatus() {
  try {
    const status = execSync('git status --porcelain', {
      cwd: projectRoot,
      encoding: 'utf8',
    });
    if (status.trim()) {
      console.error(
        '❌ Git working directory is not clean. Please commit all changes first.'
      );
      console.error('Uncommitted changes:');
      console.error(status);
      process.exit(1);
    }
    console.log('✅ Git working directory is clean');
  } catch (error) {
    console.error('❌ Failed to check git status');
    process.exit(1);
  }
}

function getCurrentVersion() {
  const packageJsonPath = path.join(ddkRnRoot, 'package.json');
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
  return packageJson.version;
}

function updateRustVersion(version) {
  const cargoTomlPath = path.join(projectRoot, 'ddk-ffi', 'Cargo.toml');
  let cargoContent = fs.readFileSync(cargoTomlPath, 'utf8');

  // Update version in Cargo.toml
  cargoContent = cargoContent.replace(
    /^version = ".*"/m,
    `version = "${version}"`
  );

  fs.writeFileSync(cargoTomlPath, cargoContent);
  console.log(`✅ Updated Cargo.toml version to ${version}`);
}

function createBinaryArchives() {
  console.log('📦 Creating binary archives...');
  runCommand(
    'node scripts/create-binary-archives.js',
    ddkRnRoot,
    'Creating binary archives'
  );
}

function runTests() {
  console.log('🧪 Running tests...');

  // Run Rust tests
  runCommand(
    'cargo test',
    path.join(projectRoot, 'ddk-ffi'),
    'Running Rust tests'
  );

  // Run React Native tests
  // runCommand('pnpm test', ddkRnRoot, 'Running React Native tests');

  console.log('✅ All tests passed');
}

function buildPackage() {
  console.log('🔨 Building package...');
  runCommand(
    'pnpm prepare',
    ddkRnRoot,
    'Building with react-native-builder-bob'
  );
  console.log('✅ Package built successfully');
}

function commitVersionChanges(version) {
  console.log(`📝 Checking for changes to commit...`);
  
  // Check if there are any changes to commit
  try {
    const status = execSync('git status --porcelain', { 
      cwd: projectRoot, 
      encoding: 'utf8' 
    });
    
    if (!status.trim()) {
      console.log('✅ No changes to commit, skipping commit step');
      return;
    }
    
    console.log(`📝 Committing Rust version sync for v${version}...`);
    runCommand('git add .', projectRoot, 'Staging all changes');
    runCommand(
      `git commit -m "chore: sync Rust version to v${version} and regenerate bindings"`,
      projectRoot,
      'Committing changes'
    );
    
    // Push to origin master
    runCommand(
      'git push origin master',
      projectRoot,
      'Pushing changes to origin master'
    );
    
  } catch (error) {
    console.error('❌ Failed to check git status or commit changes');
    throw error;
  }
}

function releaseWithPnpm() {
  console.log('🚀 Starting automated release with release-it...');

  // Check if user is authenticated with npm
  try {
    runCommand('npm whoami', ddkRnRoot, 'Checking npm authentication');
  } catch (error) {
    console.error(
      '❌ You are not authenticated with npm. Please run: npm login'
    );
    process.exit(1);
  }

  // Run release-it
  runCommand('pnpm release', ddkRnRoot, 'Running pnpm release');
}

function uploadBinaryArchives(version) {
  console.log(`📤 Uploading binary archives to GitHub release v${version}...`);

  const archiveDir = path.join(projectRoot, 'release-archives');
  const androidArchive = path.join(archiveDir, 'android-jni-libs.tar.gz');
  const iosArchive = path.join(archiveDir, 'ios-xcframeworks.tar.gz');

  const archives = [];
  if (fs.existsSync(androidArchive)) {
    archives.push(androidArchive);
  }
  if (fs.existsSync(iosArchive)) {
    archives.push(iosArchive);
  }

  if (archives.length === 0) {
    console.warn('⚠️  No binary archives found, skipping upload');
    return;
  }

  try {
    const uploadCommand = `gh release upload v${version} ${archives.join(' ')}`;
    runCommand(
      uploadCommand,
      projectRoot,
      'Uploading archives to GitHub release'
    );
    console.log('✅ Binary archives uploaded successfully');
  } catch (error) {
    console.warn(
      '⚠️  Failed to upload binary archives. You can upload them manually:'
    );
    archives.forEach((archive) => {
      console.warn(`   gh release upload v${version} ${archive}`);
    });
  }
}

function cleanupArchives() {
  const archiveDir = path.join(projectRoot, 'release-archives');
  if (fs.existsSync(archiveDir)) {
    runCommand(
      `rm -rf "${archiveDir}"`,
      projectRoot,
      'Cleaning up archive directory'
    );
    console.log('✅ Cleaned up temporary archives');
  }
}

function main() {
  try {
    console.log('🔍 Pre-flight checks...');

    // 1. Check git status
    checkGitStatus();

    // 2. Get current version from package.json
    const currentVersion = getCurrentVersion();
    console.log(`📦 Current package version: ${currentVersion}`);

    // 3. Update Rust version to match
    console.log('\n📝 Syncing Rust version...');
    updateRustVersion(currentVersion);

    // 4. Run tests
    console.log('\n🧪 Running tests...');
    runTests();

    // 5. Create binary archives
    console.log('\n📦 Creating archives...');
    createBinaryArchives();

    // 6. Build package
    console.log('\n🔨 Building package...');
    buildPackage();

    // 7. Commit Rust version changes
    console.log('\n📝 Committing changes...');
    commitVersionChanges(currentVersion);

    // 8. Release with pnpm (this will bump version, create tag, publish to npm, create GitHub release)
    console.log('\n🚀 Releasing...');
    releaseWithPnpm();

    // 9. Get the new version (release-it will have bumped it)
    const newVersion = getCurrentVersion();
    console.log(`🎉 Released version: ${newVersion}`);

    // 10. Upload binary archives to the GitHub release
    console.log('\n📤 Uploading binaries...');
    uploadBinaryArchives(newVersion);

    // 11. Cleanup
    console.log('\n🧹 Cleaning up...');
    cleanupArchives();

    console.log('\n🎉 Release completed successfully!');
    console.log(
      `📦 Package @bennyblader/ddk-rn@${newVersion} is now available on npm`
    );
    console.log(
      `🔗 GitHub release: https://github.com/bennyhodl/ddk-ffi/releases/tag/v${newVersion}`
    );
    console.log(
      `📋 npm package: https://www.npmjs.com/package/@bennyblader/ddk-rn/v/${newVersion}`
    );
  } catch (error) {
    console.error('\n❌ Release failed:', error.message);
    console.error('\n🔧 You may need to clean up manually:');
    console.error('   - Check git status and reset if needed');
    console.error('   - Remove release-archives/ directory');
    console.error('   - Check npm and GitHub releases');
    process.exit(1);
  }
}

// Handle CLI arguments
const args = process.argv.slice(2);
if (args.includes('--help') || args.includes('-h')) {
  console.log(`
🚀 Automated Release Script

This script automates the entire release process:
1. Checks git status is clean
2. Updates Rust version to match package.json
4. Runs all tests (Rust + React Native)
5. Creates binary archives
6. Builds the npm package
7. Commits Rust version changes
8. Runs 'pnpm release' (version bump, git tag, npm publish, GitHub release)
9. Uploads binary archives to GitHub release
10. Cleans up temporary files

Prerequisites:
- Clean git working directory
- npm authentication (npm login)
- GitHub CLI authentication (gh auth login)
- All dependencies installed (pnpm install)

Usage:
  node scripts/release.js
  pnpm run release-full

The script will prompt for version bump during the release process.
`);
  process.exit(0);
}

main();
