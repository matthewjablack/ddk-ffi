const ddkTs = require('./src/index.js');

console.log('Testing BigInt conversion...');
const output = {
  value: 500n,
  scriptPubkey: Buffer.alloc(22, 0)
};

console.log('Output object:', output);
console.log('Value type:', typeof output.value);
console.log('Value:', output.value);

try {
  const result = ddkTs.isDustOutput(output);
  console.log('Result:', result);
} catch (error) {
  console.error('Error:', error);
}
