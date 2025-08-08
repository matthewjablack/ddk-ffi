#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log('📦 Creating binary archives for GitHub release...');

const projectRoot = path.join(__dirname, '..', '..');
const ddkRnRoot = path.join(__dirname, '..');

// Paths to binary directories
const androidLibsDir = path.join(ddkRnRoot, 'android', 'src', 'main', 'jniLibs');
const iosDir = path.join(ddkRnRoot, 'ios');

// Output directory for archives
const archiveDir = path.join(projectRoot, 'release-archives');

function ensureDir(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

function createTarGz(sourceDir, archivePath, baseDir = null) {
  try {
    const tarCommand = baseDir 
      ? `tar -czf "${archivePath}" -C "${path.dirname(sourceDir)}" "${path.basename(sourceDir)}"`
      : `tar -czf "${archivePath}" -C "${sourceDir}" .`;
    
    console.log(`📂 Creating ${path.basename(archivePath)}...`);
    execSync(tarCommand, { stdio: 'inherit' });
    console.log(`✅ Created ${archivePath}`);
    
    // Show archive size
    const stats = fs.statSync(archivePath);
    const sizeMB = (stats.size / 1024 / 1024).toFixed(2);
    console.log(`   Size: ${sizeMB} MB`);
  } catch (error) {
    throw new Error(`Failed to create archive ${archivePath}: ${error.message}`);
  }
}

function main() {
  try {
    ensureDir(archiveDir);
    
    // Create Android JNI libraries archive
    if (fs.existsSync(androidLibsDir)) {
      const androidArchive = path.join(archiveDir, 'android-jni-libs.tar.gz');
      createTarGz(androidLibsDir, androidArchive);
    } else {
      console.warn('⚠️  Android JNI libraries not found, skipping android-jni-libs.tar.gz');
    }
    
    // Create iOS XCFrameworks archive
    const xcframeworkFiles = fs.readdirSync(iosDir).filter(file => file.endsWith('.xcframework'));
    if (xcframeworkFiles.length > 0) {
      const iosArchive = path.join(archiveDir, 'ios-xcframeworks.tar.gz');
      
      // Create temporary directory with just XCFrameworks
      const tempDir = path.join(archiveDir, 'temp-ios');
      ensureDir(tempDir);
      
      xcframeworkFiles.forEach(framework => {
        const src = path.join(iosDir, framework);
        const dest = path.join(tempDir, framework);
        execSync(`cp -R "${src}" "${dest}"`, { stdio: 'inherit' });
      });
      
      createTarGz(tempDir, iosArchive);
      
      // Clean up temp directory
      execSync(`rm -rf "${tempDir}"`, { stdio: 'inherit' });
    } else {
      console.warn('⚠️  iOS XCFrameworks not found, skipping ios-xcframeworks.tar.gz');
    }
    
    console.log('🎉 Binary archives created successfully!');
    console.log(`📁 Archives location: ${archiveDir}`);
    console.log('');
    console.log('📋 Next steps:');
    console.log('   1. Upload these archives to your GitHub release');
    console.log('   2. The postinstall script will automatically download them');
    
  } catch (error) {
    console.error('❌ Failed to create binary archives:', error.message);
    process.exit(1);
  }
}

main();