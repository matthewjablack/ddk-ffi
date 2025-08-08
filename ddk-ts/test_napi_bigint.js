// Test to understand NAPI-RS BigInt behavior
const testValues = [
  500n,
  -500n,
  0n,
  1n,
  -1n,
  BigInt(Number.MAX_SAFE_INTEGER),
  BigInt(-Number.MAX_SAFE_INTEGER)
];

console.log('Testing BigInt values:');
for (const val of testValues) {
  console.log(`Value: ${val}, Sign: ${val < 0n ? 'negative' : 'positive'}`);
}