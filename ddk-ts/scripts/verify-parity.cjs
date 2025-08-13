#!/usr/bin/env node

const fs = require('fs')
const path = require('path')

// Parse UDL to get all functions
function parseUDLFunctions(udlPath) {
  const content = fs.readFileSync(udlPath, 'utf8')
  // Match function declarations in UDL
  const functionRegex = /^\s*(?:\[Throws=\w+\])?\s*(?:sequence<)?[\w<>]+>?\s+(\w+)\s*\(/gm
  const functions = []

  let match
  while ((match = functionRegex.exec(content)) !== null) {
    // Skip the namespace declaration
    if (match[1] !== 'ddk_ffi') {
      functions.push(match[1])
    }
  }

  return functions
}

// Parse Rust lib.rs to get all NAPI functions
function parseNAPIFunctions(libPath) {
  const content = fs.readFileSync(libPath, 'utf8')
  // Match #[napi] decorated functions
  const functionRegex = /#\[napi\]\s*pub\s+fn\s+(\w+)/g
  const functions = []

  let match
  while ((match = functionRegex.exec(content)) !== null) {
    functions.push(match[1])
  }

  return functions
}

// Convert snake_case to camelCase
function toCamelCase(str) {
  return str.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase())
}

// Convert camelCase to snake_case
function toSnakeCase(str) {
  return str.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`).replace(/^_/, '')
}

// Main verification
function verifyParity() {
  const udlPath = path.join(__dirname, '../../ddk-ffi/src/ddk_ffi.udl')
  const libPath = path.join(__dirname, '../src/lib.rs')
  const testPath = path.join(__dirname, '../__test__/index.spec.ts')

  console.log('🔍 Verifying NAPI-RS and UniFFI parity...\n')

  // Parse UDL functions
  const udlFunctions = parseUDLFunctions(udlPath)
  console.log(`📋 Found ${udlFunctions.length} functions in UDL:`)
  udlFunctions.forEach((fn) => console.log(`   - ${fn}`))
  console.log()

  // Parse NAPI functions
  const napiFunctions = parseNAPIFunctions(libPath)
  console.log(`🦀 Found ${napiFunctions.length} functions in NAPI lib.rs:`)
  napiFunctions.forEach((fn) => console.log(`   - ${fn}`))
  console.log()

  // Check for missing functions
  const missingInNAPI = []
  const extraInNAPI = []

  // Convert NAPI functions to snake_case for comparison
  const napiFunctionsSnake = napiFunctions.map(toSnakeCase)

  // Check each UDL function
  for (const udlFunc of udlFunctions) {
    if (!napiFunctionsSnake.includes(udlFunc)) {
      missingInNAPI.push(udlFunc)
    }
  }

  // Check for extra functions in NAPI
  for (const napiFunc of napiFunctionsSnake) {
    if (!udlFunctions.includes(napiFunc)) {
      extraInNAPI.push(napiFunc)
    }
  }

  // Parse test file to check test coverage
  const testContent = fs.readFileSync(testPath, 'utf8')
  const testedFunctions = new Set()

  // Look for function calls in tests (camelCase)
  const callRegex = /ddk\.(\w+)\(/g
  let match
  while ((match = callRegex.exec(testContent)) !== null) {
    testedFunctions.add(toSnakeCase(match[1]))
  }

  // Check test coverage
  const untestedFunctions = []
  for (const udlFunc of udlFunctions) {
    if (!testedFunctions.has(udlFunc)) {
      untestedFunctions.push(udlFunc)
    }
  }

  // Report results
  console.log('📊 Parity Check Results:\n')

  if (missingInNAPI.length > 0) {
    console.error('❌ Functions missing in NAPI implementation:')
    missingInNAPI.forEach((fn) => console.error(`   - ${fn} (should be ${toCamelCase(fn)} in NAPI)`))
    console.log()
  }

  if (extraInNAPI.length > 0) {
    console.warn('⚠️  Extra functions in NAPI (not in UDL):')
    extraInNAPI.forEach((fn) => console.warn(`   - ${fn}`))
    console.log()
  }

  if (untestedFunctions.length > 0) {
    console.warn('⚠️  Functions without test coverage:')
    untestedFunctions.forEach((fn) => console.warn(`   - ${fn} (${toCamelCase(fn)} in tests)`))
    console.log()
  }

  // Final verdict
  if (missingInNAPI.length === 0 && extraInNAPI.length === 0) {
    console.log('✅ Perfect parity! All UDL functions are implemented in NAPI.')
  } else {
    console.log('⚠️  Parity issues found. Please review the discrepancies above.')
    process.exit(1)
  }

  if (untestedFunctions.length === 0) {
    console.log('✅ All functions have test coverage!')
  } else {
    console.log('⚠️  Some functions lack test coverage.')
  }

  console.log('\n📈 Summary:')
  console.log(`   UDL Functions: ${udlFunctions.length}`)
  console.log(`   NAPI Functions: ${napiFunctions.length}`)
  console.log(
    `   Test Coverage: ${(((udlFunctions.length - untestedFunctions.length) / udlFunctions.length) * 100).toFixed(1)}%`,
  )
}

// Run verification
try {
  verifyParity()
} catch (error) {
  console.error('❌ Error during verification:', error.message)
  process.exit(1)
}
