#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

// Get package version to determine which release to download
// __dirname is ddk-rn/scripts/, so package.json is at ../package.json
const packageJsonPath = path.join(__dirname, '..', 'package.json');
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
const version = packageJson.version;

const GITHUB_REPO = 'bennyhodl/ddk-ffi';
const RELEASE_TAG = `v${version}`;

// Detect platform and architecture
const platform = process.platform;
const arch = process.arch;

console.log(`📦 Installing native binaries for ${platform}-${arch}...`);

// Create directories if they don't exist
const androidLibsDir = path.join(__dirname, '..', 'android', 'src', 'main', 'jniLibs');
const iosFrameworksDir = path.join(__dirname, '..', 'ios');

function ensureDir(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    console.log(`⬇️  Downloading ${url}`);
    
    const file = fs.createWriteStream(dest);
    
    https.get(url, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        // Follow redirect
        return downloadFile(response.headers.location, dest).then(resolve).catch(reject);
      }
      
      if (response.statusCode !== 200) {
        reject(new Error(`Failed to download: ${response.statusCode} ${response.statusMessage}`));
        return;
      }
      
      response.pipe(file);
      
      file.on('finish', () => {
        file.close();
        console.log(`✅ Downloaded ${path.basename(dest)}`);
        resolve();
      });
    }).on('error', (err) => {
      fs.unlink(dest, () => {}); // Delete partial file
      reject(err);
    });
  });
}

function extractTarGz(tarPath, extractDir) {
  try {
    console.log(`📂 Extracting ${path.basename(tarPath)}...`);
    execSync(`tar -xzf "${tarPath}" -C "${extractDir}"`, { stdio: 'inherit' });
    console.log(`✅ Extracted to ${extractDir}`);
  } catch (error) {
    throw new Error(`Failed to extract ${tarPath}: ${error.message}`);
  }
}

async function downloadAndExtractBinaries() {
  try {
    // Download Android JNI libraries
    const androidArchiveUrl = `https://github.com/${GITHUB_REPO}/releases/download/${RELEASE_TAG}/android-jni-libs.tar.gz`;
    const androidTarPath = path.join(__dirname, '..', 'android-jni-libs.tar.gz');
    
    ensureDir(androidLibsDir);
    await downloadFile(androidArchiveUrl, androidTarPath);
    extractTarGz(androidTarPath, path.join(__dirname, '..', 'android', 'src', 'main'));
    fs.unlinkSync(androidTarPath); // Clean up
    
    // Download iOS XCFramework (only on macOS for iOS development)
    if (platform === 'darwin') {
      const iosArchiveUrl = `https://github.com/${GITHUB_REPO}/releases/download/${RELEASE_TAG}/ios-xcframeworks.tar.gz`;
      const iosTarPath = path.join(__dirname, '..', 'ios-xcframeworks.tar.gz');
      
      ensureDir(iosFrameworksDir);
      await downloadFile(iosArchiveUrl, iosTarPath);
      extractTarGz(iosTarPath, iosFrameworksDir);
      fs.unlinkSync(iosTarPath); // Clean up
    } else {
      console.log('⚠️  Skipping iOS binaries (not on macOS)');
    }
    
    console.log('🎉 All native binaries installed successfully!');
    
  } catch (error) {
    console.error('❌ Failed to install native binaries:', error.message);
    console.error('');
    console.error('📋 Manual installation:');
    console.error(`   1. Download binaries from: https://github.com/${GITHUB_REPO}/releases/tag/${RELEASE_TAG}`);
    console.error('   2. Extract android-jni-libs.tar.gz to android/src/main/');
    console.error('   3. Extract ios-xcframeworks.tar.gz to ios/ (macOS only)');
    console.error('');
    
    // Don't fail the installation - let users manually install if needed
    process.exit(0);
  }
}

// Check if binaries already exist (skip if they do)
const androidLibExists = fs.existsSync(path.join(androidLibsDir, 'arm64-v8a'));
const iosFrameworkExists = fs.existsSync(path.join(iosFrameworksDir, 'DdkFFI.xcframework'));

if (androidLibExists && (platform !== 'darwin' || iosFrameworkExists)) {
  console.log('✅ Native binaries already installed, skipping download.');
  process.exit(0);
}

// Only download if we're not in a CI environment or explicit opt-in
if (process.env.CI && !process.env.DOWNLOAD_BINARIES) {
  console.log('⚠️  Skipping binary download in CI environment.');
  console.log('   Set DOWNLOAD_BINARIES=1 to force download in CI.');
  process.exit(0);
}

downloadAndExtractBinaries();