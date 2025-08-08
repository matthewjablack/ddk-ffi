const ddk = require('./src/index.js');

console.log('Testing BigInt sign behavior in NAPI-RS:');
console.log('Positive 500n:', ddk.testBigintSign(500n));
console.log('Negative -500n:', ddk.testBigintSign(-500n));
console.log('Zero 0n:', ddk.testBigintSign(0n));
console.log('One 1n:', ddk.testBigintSign(1n));
