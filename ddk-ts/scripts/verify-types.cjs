#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Parse UDL to get all types (dictionaries and enums)
function parseUDLTypes(udlPath) {
  const content = fs.readFileSync(udlPath, 'utf8');
  const types = {
    dictionaries: [],
    enums: []
  };
  
  // Match dictionary declarations
  const dictRegex = /^dictionary\s+(\w+)\s*{/gm;
  let match;
  while ((match = dictRegex.exec(content)) !== null) {
    types.dictionaries.push(match[1]);
  }
  
  // Match enum declarations
  const enumRegex = /^(?:\[Error\])?\s*enum\s+(\w+)\s*{/gm;
  while ((match = enumRegex.exec(content)) !== null) {
    types.enums.push(match[1]);
  }
  
  return types;
}

// Parse TypeScript types.rs to get all NAPI types
function parseNAPITypes(typesPath) {
  const content = fs.readFileSync(typesPath, 'utf8');
  const types = [];
  
  // Match #[napi(object)] decorated structs
  const structRegex = /#\[napi\(object\)\]\s*(?:#\[derive[^\]]+\])?\s*pub\s+struct\s+(\w+)/g;
  let match;
  while ((match = structRegex.exec(content)) !== null) {
    types.push(match[1]);
  }
  
  return types;
}

// Main verification
function verifyTypes() {
  const udlPath = path.join(__dirname, '../../ddk-ffi/src/ddk_ffi.udl');
  const typesPath = path.join(__dirname, '../src/types.rs');
  
  console.log('🔍 Verifying NAPI-RS type definitions...\n');
  
  // Parse UDL types
  const udlTypes = parseUDLTypes(udlPath);
  console.log(`📋 Found ${udlTypes.dictionaries.length} dictionaries in UDL:`);
  udlTypes.dictionaries.forEach(type => console.log(`   - ${type}`));
  console.log();
  
  console.log(`📋 Found ${udlTypes.enums.length} enums in UDL:`);
  udlTypes.enums.forEach(type => console.log(`   - ${type}`));
  console.log();
  
  // Parse NAPI types
  const napiTypes = parseNAPITypes(typesPath);
  console.log(`🦀 Found ${napiTypes.length} types in NAPI types.rs:`);
  napiTypes.forEach(type => console.log(`   - ${type}`));
  console.log();
  
  // Check for missing types
  const missingTypes = [];
  for (const udlType of udlTypes.dictionaries) {
    if (!napiTypes.includes(udlType)) {
      missingTypes.push(udlType);
    }
  }
  
  // Note: Enums might be handled differently in NAPI
  // DLCError is typically handled as Error types, not as a separate enum
  
  // Report results
  console.log('📊 Type Verification Results:\n');
  
  if (missingTypes.length > 0) {
    console.error('❌ Types missing in NAPI implementation:');
    missingTypes.forEach(type => console.error(`   - ${type}`));
    console.log();
    process.exit(1);
  } else {
    console.log('✅ All UDL dictionary types are defined in NAPI!');
  }
  
  console.log('\n📈 Summary:');
  console.log(`   UDL Dictionaries: ${udlTypes.dictionaries.length}`);
  console.log(`   UDL Enums: ${udlTypes.enums.length}`);
  console.log(`   NAPI Types: ${napiTypes.length}`);
}

// Run verification
try {
  verifyTypes();
} catch (error) {
  console.error('❌ Error during type verification:', error.message);
  process.exit(1);
}