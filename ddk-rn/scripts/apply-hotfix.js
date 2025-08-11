#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const ddkRnRoot = path.join(__dirname, '..');
const projectRoot = path.join(ddkRnRoot, '..');

function runCommand(command, cwd = projectRoot, options = {}) {
  const { silent = false } = options;
  try {
    const result = execSync(command, {
      cwd,
      stdio: silent ? 'pipe' : 'inherit',
      encoding: 'utf8',
    });
    return result;
  } catch (error) {
    if (!silent) {
      console.error(`❌ Command failed: ${command}`);
    }
    throw error;
  }
}

function applyHotFix() {
  console.log('🔧 Applying post-build hot fixes...');

  const cppFile = path.join(ddkRnRoot, 'cpp', 'bennyblader-ddk-rn.cpp');
  const podspecFile = path.join(ddkRnRoot, 'DdkRn.podspec');

  // Fix 1: C++ include path
  if (fs.existsSync(cppFile)) {
    let cppContent = fs.readFileSync(cppFile, 'utf8');
    if (cppContent.includes('#include "/ddk_ffi.hpp"')) {
      cppContent = cppContent.replace(
        '#include "/ddk_ffi.hpp"',
        '#include "ddk_ffi.hpp"'
      );
      fs.writeFileSync(cppFile, cppContent);
      console.log('   ✅ Fixed include path in bennyblader-ddk-rn.cpp');
    } else {
      console.log('   ✅ Include path already correct');
    }
  }

  // Fix 2: Add back xcconfig LIBRARY_SEARCH_PATHS to podspec (gets removed by uniffi build)
  if (fs.existsSync(podspecFile)) {
    let podspecContent = fs.readFileSync(podspecFile, 'utf8');

    // Check if xcconfig is missing
    if (
      !podspecContent.includes('s.xcconfig = {') &&
      !podspecContent.includes('LIBRARY_SEARCH_PATHS')
    ) {
      // Find the line with s.vendored_frameworks and add xcconfig after it
      const vendoredFrameworksLine =
        's.vendored_frameworks = "ios/DdkRn.xcframework"';
      if (podspecContent.includes(vendoredFrameworksLine)) {
        const xconfigBlock = `  s.xcconfig = {
    'LIBRARY_SEARCH_PATHS' => '$(SRCROOT)/../node_modules/@bennyblader/ddk-rn/ios/DdkRn.xcframework/ios-arm64-simulator $(SRCROOT)/../node_modules/@bennyblader/ddk-rn/ios/DdkRn.xcframework/ios-arm64 $(SRCROOT)/../node_modules/@bennyblader/ddk-rn/ios/DdkRn.xcframework/ios-x86_64-simulator $(SRCROOT)/../node_modules/@bennyblader/ddk-rn/ios/DdkRn.xcframework/ios-x86_64'
  }`;

        podspecContent = podspecContent.replace(
          vendoredFrameworksLine,
          vendoredFrameworksLine + '\n' + xconfigBlock
        );

        fs.writeFileSync(podspecFile, podspecContent);
        console.log(
          '   ✅ Added back xcconfig LIBRARY_SEARCH_PATHS to DdkRn.podspec'
        );
      } else {
        console.log(
          '   ⚠️  Could not find vendored_frameworks line in podspec'
        );
      }
    } else {
      console.log('   ✅ DdkRn.podspec already has xcconfig');
    }
  }

  console.log('   ✅ Hot fixes applied and files unstaged');
  console.log();
}

// Export for use in other scripts
module.exports = { applyHotFix };

// Allow running directly
if (require.main === module) {
  console.log('🔧 Running hot fix script...\n');
  applyHotFix();
  console.log('✅ Hot fix completed!\n');
}
