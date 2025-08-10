#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const packageRoot = path.join(__dirname, '..');
const ddkFfiRoot = path.join(packageRoot, 'ddk-ffi');

console.log('📦 DDK-RN Post-install: Building native libraries...');

// Check if uniffi-bindgen-react-native is available (prefer npx)
function hasUniffiBingen() {
  try {
    execSync('npx uniffi-bindgen-react-native --help', { stdio: 'ignore' });
    return true;
  } catch (error) {
    try {
      execSync('uniffi-bindgen-react-native --help', { stdio: 'ignore' });
      return true;
    } catch (error2) {
      return false;
    }
  }
}

// Get the uniffi command (prefer npx)
function getUniffiCommand() {
  try {
    execSync('npx uniffi-bindgen-react-native --help', { stdio: 'ignore' });
    return 'npx uniffi-bindgen-react-native';
  } catch (error) {
    return 'uniffi-bindgen-react-native';
  }
}

// Create directories if they don't exist
function ensureDir(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

// Build iOS libraries and create XCFramework
function buildIOS() {
  const platform = process.platform;
  
  if (platform !== 'darwin') {
    console.log('\n⚠️  Skipping iOS build (not on macOS)');
    return;
  }
  
  console.log('\n🍎 Building iOS libraries...');
  const uniffiCmd = getUniffiCommand();
  
  // Update ubrn.config.yaml to point to the included Rust source
  updateConfigFile();
  
  try {
    const cmd = `${uniffiCmd} build ios --and-generate`;
    console.log(`Running: ${cmd}`);
    execSync(cmd, { 
      stdio: 'inherit',
      cwd: packageRoot 
    });
    console.log('✅ iOS libraries built');
  } catch (error) {
    throw new Error(`Failed to build iOS libraries: ${error.message}`);
  }
}

// Build Android libraries
function buildAndroid() {
  console.log('\n📱 Building Android libraries...');
  
  // Check if Android NDK is available
  if (!process.env.ANDROID_NDK_ROOT && !process.env.NDK_HOME) {
    console.log('⚠️  Android NDK not found. Skipping Android build.');
    console.log('   Set ANDROID_NDK_ROOT or NDK_HOME to build Android libraries.');
    return;
  }
  
  const uniffiCmd = getUniffiCommand();
  
  // Update ubrn.config.yaml to point to the included Rust source
  updateConfigFile();
  
  try {
    const cmd = `${uniffiCmd} build android --and-generate`;
    console.log(`Running: ${cmd}`);
    execSync(cmd, { 
      stdio: 'inherit',
      cwd: packageRoot 
    });
    console.log('✅ Android libraries built');
  } catch (error) {
    console.warn(`⚠️  Android build failed: ${error.message}`);
    console.warn('   This may be due to missing Android NDK or Rust toolchains.');
  }
}

// Update the ubrn.config.yaml to use the local Rust source
function updateConfigFile() {
  const configPath = path.join(packageRoot, 'ubrn.config.yaml');
  
  if (fs.existsSync(configPath)) {
    let config = fs.readFileSync(configPath, 'utf8');
    
    // Update the Rust directory to point to the included ddk-ffi
    if (!config.includes('directory: ddk-ffi')) {
      config = config.replace(
        /rust:\s*\n\s*directory:\s*[^\n]+/,
        'rust:\n  directory: ddk-ffi'
      );
      fs.writeFileSync(configPath, config);
      console.log('📝 Updated config to use included Rust source');
    }
  }
}

// Fix the C++ include path issue if needed
function fixCppIncludePath() {
  const cppFile = path.join(packageRoot, 'cpp', 'bennyblader-ddk-rn.cpp');
  if (fs.existsSync(cppFile)) {
    let content = fs.readFileSync(cppFile, 'utf8');
    if (content.includes('#include "/ddk_ffi.hpp"')) {
      content = content.replace('#include "/ddk_ffi.hpp"', '#include "ddk_ffi.hpp"');
      fs.writeFileSync(cppFile, content);
      console.log('🔧 Fixed include path in C++ bindings');
    }
  }
}

// Verify all required files are present
function verifyAllFiles() {
  console.log('\n🔍 Verifying installation...');
  
  const requiredFiles = [
    // Core JSI bindings (shipped with package)
    'src/ddk_ffi.ts',
    'src/ddk_ffi-ffi.ts',
    'cpp/ddk_ffi.hpp',
    'cpp/ddk_ffi.cpp',
    // Generated turbo module files (shipped with package)
    'src/NativeDdkRn.ts',
    'src/index.tsx',
    'cpp/bennyblader-ddk-rn.cpp',
    'cpp/bennyblader-ddk-rn.h',
    // Rust source (shipped with package)
    'ddk-ffi/Cargo.toml',
    'ddk-ffi/src/lib.rs',
    'ddk-ffi/src/ddk_ffi.udl'
  ];
  
  const platform = process.platform;
  let allFilesPresent = true;
  
  // Check required files
  console.log('📋 Checking required files...');
  for (const file of requiredFiles) {
    const filePath = path.join(packageRoot, file);
    if (fs.existsSync(filePath)) {
      console.log(`  ✅ ${file}`);
    } else {
      console.error(`  ❌ Missing: ${file}`);
      allFilesPresent = false;
    }
  }
  
  // Check platform-specific files (generated during postinstall)
  if (platform === 'darwin') {
    console.log('📋 Checking iOS framework...');
    const iosFramework = path.join(packageRoot, 'ios', 'DdkRn.xcframework', 'Info.plist');
    if (fs.existsSync(iosFramework)) {
      console.log(`  ✅ ios/DdkRn.xcframework`);
    } else {
      console.log(`  ⚠️  iOS framework not built yet (will be built now)`);
    }
  }
  
  console.log('📋 Checking Android libraries...');
  const androidLibs = [
    'android/src/main/arm64-v8a/libddk_ffi.a',
    'android/src/main/armeabi-v7a/libddk_ffi.a',
    'android/src/main/x86/libddk_ffi.a',
    'android/src/main/x86_64/libddk_ffi.a'
  ];
  
  let androidLibsPresent = 0;
  for (const lib of androidLibs) {
    const libPath = path.join(packageRoot, lib);
    if (fs.existsSync(libPath)) {
      androidLibsPresent++;
    }
  }
  
  if (androidLibsPresent === 0) {
    console.log('  ⚠️  Android libraries not built yet (may be due to missing NDK)');
  } else {
    console.log(`  ✅ Found ${androidLibsPresent}/${androidLibs.length} Android libraries`);
  }
  
  return allFilesPresent;
}

async function main() {
  // Skip in CI unless explicitly requested
  if (process.env.CI && !process.env.BUILD_NATIVE_LIBS) {
    console.log('⚠️  Skipping native library builds in CI environment.');
    console.log('   Set BUILD_NATIVE_LIBS=1 to force builds in CI.');
    process.exit(0);
  }
  
  // Check prerequisites
  if (!hasUniffiBingen()) {
    console.error('❌ uniffi-bindgen-react-native not found!');
    console.error('   Install it with: npm install -g uniffi-bindgen-react-native');
    console.error('   Or add it as a dependency in your project.');
    process.exit(1);
  }
  
  // Check if Rust source is included
  if (!fs.existsSync(ddkFfiRoot)) {
    console.error('❌ Rust source not found in package!');
    console.error(`   Expected at: ${ddkFfiRoot}`);
    console.error('   This indicates a problem with the NPM package.');
    process.exit(1);
  }
  
  // Verify source files are present (should be included in NPM package)
  const sourceFiles = [
    'src/ddk_ffi.ts',
    'src/ddk_ffi-ffi.ts', 
    'src/NativeDdkRn.ts',
    'src/index.tsx',
    'cpp/ddk_ffi.hpp',
    'cpp/ddk_ffi.cpp',
    'cpp/bennyblader-ddk-rn.cpp',
    'cpp/bennyblader-ddk-rn.h',
    'ddk-ffi/Cargo.toml',
    'ddk-ffi/src/lib.rs',
    'ddk-ffi/src/ddk_ffi.udl'
  ];
  
  console.log('🔍 Checking source files...');
  for (const file of sourceFiles) {
    const filePath = path.join(packageRoot, file);
    if (!fs.existsSync(filePath)) {
      console.error(`❌ Missing source file: ${file}`);
      console.error('   This indicates a problem with the NPM package.');
      process.exit(1);
    }
  }
  console.log('✅ All source files present');
  
  try {
    // Fix include path if needed
    fixCppIncludePath();
    
    // Build native libraries only (turbo module already generated)
    buildIOS();
    buildAndroid();
    
    // Verify all files are present
    const allFilesPresent = verifyAllFiles();
    
    if (!allFilesPresent) {
      console.error('\n❌ Some required files are missing!');
      console.error('The installation may have failed.');
      process.exit(1);
    }
    
    console.log('\n✅ Installation completed successfully!');
    console.log('🎉 DDK-RN is ready to use!\n');
    
  } catch (error) {
    console.error('\n❌ Failed to complete installation:', error.message);
    console.error('');
    console.error('📋 This may be due to:');
    console.error('   - Missing uniffi-bindgen-react-native (install globally)');
    console.error('   - Missing Android NDK (for Android builds)');
    console.error('   - Missing Xcode/iOS toolchain (for iOS builds on macOS)');
    console.error('   - Missing Rust toolchain');
    console.error('');
    console.error('Report issues at: https://github.com/bennyhodl/ddk-ffi/issues');
    process.exit(1);
  }
}

// Run the main function
main().catch(error => {
  console.error('❌ Unexpected error:', error);
  process.exit(1);
});