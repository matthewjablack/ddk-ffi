#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

console.log('📦 DDK-RN Post-install: Verifying native bindings...');

const packageRoot = path.join(__dirname, '..');

// Verify that all necessary files are present
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

let allFilesPresent = true;
const platform = process.platform;

for (const [category, files] of Object.entries(requiredFiles)) {
  // Skip iOS files on non-macOS platforms
  if (category === 'iOS framework' && platform !== 'darwin') {
    console.log(`⚠️  Skipping ${category} verification (not on macOS)`);
    continue;
  }
  
  console.log(`\n🔍 Checking ${category}...`);
  
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

if (!allFilesPresent) {
  console.error('\n❌ Some required files are missing!');
  console.error('This package may not have been published correctly.');
  console.error('Please report this issue at: https://github.com/bennyhodl/ddk-ffi/issues');
  process.exit(1);
}

console.log('\n✅ All required bindings are present!');
console.log('🎉 DDK-RN is ready to use!\n');

// Fix the C++ include path issue if needed
const cppFile = path.join(packageRoot, 'cpp', 'bennyblader-ddk-rn.cpp');
if (fs.existsSync(cppFile)) {
  let content = fs.readFileSync(cppFile, 'utf8');
  if (content.includes('#include "/ddk_ffi.hpp"')) {
    content = content.replace('#include "/ddk_ffi.hpp"', '#include "ddk_ffi.hpp"');
    fs.writeFileSync(cppFile, content);
    console.log('🔧 Fixed include path in C++ bindings');
  }
}