#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const packageRoot = path.join(__dirname, '..');
const packageJson = JSON.parse(fs.readFileSync(path.join(packageRoot, 'package.json'), 'utf8'));
const version = packageJson.version;

const GITHUB_REPO = 'bennyhodl/ddk-ffi';
const RELEASE_TAG = `v${version}`;

console.log(`📦 DDK-RN Post-install: Setting up native binaries for v${version}...`);

// Define all required files for verification
const requiredFiles = {
  'TypeScript bindings': [
    'src/ddk_ffi.ts',
    'src/ddk_ffi-ffi.ts',
    'src/NativeDdkRn.ts',
    'src/index.tsx'
  ],
  'C++ bindings': [
    'cpp/ddk_ffi.hpp',
    'cpp/ddk_ffi.cpp',
    'cpp/bennyblader-ddk-rn.h',
    'cpp/bennyblader-ddk-rn.cpp'
  ],
  'Android libraries': [
    'android/src/main/arm64-v8a/libddk_ffi.a',
    'android/src/main/armeabi-v7a/libddk_ffi.a',
    'android/src/main/x86/libddk_ffi.a',
    'android/src/main/x86_64/libddk_ffi.a'
  ],
  'iOS framework': [
    'ios/DdkRn.xcframework/Info.plist',
    'ios/DdkRn.xcframework/ios-arm64/libddk_ffi.a',
    'ios/DdkRn.xcframework/ios-arm64-simulator/libddk_ffi.a'
  ],
  'JavaScript modules': [
    'lib/commonjs/index.js',
    'lib/module/index.js'
  ]
};

// Create directories if they don't exist
function ensureDir(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    console.log(`⬇️  Downloading from ${url}...`);
    
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

function verifyAllFiles() {
  console.log('\n🔍 Verifying all required files...');
  
  let allFilesPresent = true;
  const platform = process.platform;
  
  for (const [category, files] of Object.entries(requiredFiles)) {
    // Skip iOS files on non-macOS platforms
    if (category === 'iOS framework' && platform !== 'darwin') {
      console.log(`\n⚠️  Skipping ${category} verification (not on macOS)`);
      continue;
    }
    
    console.log(`\n📋 Checking ${category}...`);
    
    for (const file of files) {
      const filePath = path.join(packageRoot, file);
      if (fs.existsSync(filePath)) {
        console.log(`  ✅ ${file}`);
      } else {
        console.error(`  ❌ Missing: ${file}`);
        allFilesPresent = false;
      }
    }
  }
  
  return allFilesPresent;
}

async function downloadAndExtractBinaries() {
  const platform = process.platform;
  
  try {
    // Download Android native libraries
    const androidLibsDir = path.join(packageRoot, 'android', 'src', 'main');
    ensureDir(androidLibsDir);
    
    const androidArchiveUrl = `https://github.com/${GITHUB_REPO}/releases/download/${RELEASE_TAG}/android-jni-libs.tar.gz`;
    const androidTarPath = path.join(packageRoot, 'android-jni-libs.tar.gz');
    
    console.log('\n📱 Setting up Android native libraries...');
    await downloadFile(androidArchiveUrl, androidTarPath);
    extractTarGz(androidTarPath, androidLibsDir);
    fs.unlinkSync(androidTarPath); // Clean up tar file
    
    // Download iOS XCFramework (only on macOS)
    if (platform === 'darwin') {
      const iosFrameworksDir = path.join(packageRoot, 'ios');
      ensureDir(iosFrameworksDir);
      
      const iosArchiveUrl = `https://github.com/${GITHUB_REPO}/releases/download/${RELEASE_TAG}/ios-xcframeworks.tar.gz`;
      const iosTarPath = path.join(packageRoot, 'ios-xcframeworks.tar.gz');
      
      console.log('\n🍎 Setting up iOS framework...');
      await downloadFile(iosArchiveUrl, iosTarPath);
      extractTarGz(iosTarPath, iosFrameworksDir);
      fs.unlinkSync(iosTarPath); // Clean up tar file
    } else {
      console.log('\n⚠️  Skipping iOS framework (not on macOS)');
    }
    
  } catch (error) {
    console.error('\n❌ Failed to download native binaries:', error.message);
    console.error('');
    console.error('📋 Manual installation instructions:');
    console.error(`   1. Download binaries from: https://github.com/${GITHUB_REPO}/releases/tag/${RELEASE_TAG}`);
    console.error('   2. Extract android-jni-libs.tar.gz to android/src/main/');
    console.error('   3. Extract ios-xcframeworks.tar.gz to ios/ (macOS only)');
    console.error('');
    console.error('Or report this issue at: https://github.com/bennyhodl/ddk-ffi/issues');
    
    // Don't fail the installation completely
    process.exit(0);
  }
}

async function main() {
  const platform = process.platform;
  
  // Skip in CI unless explicitly requested
  if (process.env.CI && !process.env.DOWNLOAD_BINARIES) {
    console.log('⚠️  Skipping binary download in CI environment.');
    console.log('   Set DOWNLOAD_BINARIES=1 to force download in CI.');
    process.exit(0);
  }
  
  // Check if binaries already exist
  const androidLibExists = fs.existsSync(path.join(packageRoot, 'android', 'src', 'main', 'arm64-v8a', 'libddk_ffi.a'));
  const iosFrameworkExists = fs.existsSync(path.join(packageRoot, 'ios', 'DdkRn.xcframework'));
  
  let needsDownload = false;
  
  if (!androidLibExists) {
    console.log('📱 Android libraries not found, will download...');
    needsDownload = true;
  }
  
  if (platform === 'darwin' && !iosFrameworkExists) {
    console.log('🍎 iOS framework not found, will download...');
    needsDownload = true;
  }
  
  if (needsDownload) {
    await downloadAndExtractBinaries();
  } else {
    console.log('✅ Native binaries already present.');
  }
  
  // Fix the C++ include path issue if needed
  const cppFile = path.join(packageRoot, 'cpp', 'bennyblader-ddk-rn.cpp');
  if (fs.existsSync(cppFile)) {
    let content = fs.readFileSync(cppFile, 'utf8');
    if (content.includes('#include "/ddk_ffi.hpp"')) {
      content = content.replace('#include "/ddk_ffi.hpp"', '#include "ddk_ffi.hpp"');
      fs.writeFileSync(cppFile, content);
      console.log('\n🔧 Fixed include path in C++ bindings');
    }
  }
  
  // Verify all files are present
  const allFilesPresent = verifyAllFiles();
  
  if (!allFilesPresent) {
    console.error('\n❌ Some required files are missing!');
    console.error('The package installation may have failed.');
    console.error('Please report this issue at: https://github.com/bennyhodl/ddk-ffi/issues');
    process.exit(1);
  }
  
  console.log('\n✅ All required files verified!');
  console.log('🎉 DDK-RN is ready to use!\n');
}

// Run the main function
main().catch(error => {
  console.error('❌ Unexpected error:', error);
  process.exit(1);
});