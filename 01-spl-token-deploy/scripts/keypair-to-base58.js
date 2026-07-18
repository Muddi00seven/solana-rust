// Converts a Solana CLI JSON keypair file (the array-of-numbers format that
// `solana-keygen new --outfile x.json` produces) into the base58 string that
// this project's .env file expects for DEPLOYER_PRIVATE_KEY.
//
// Usage:
//   node scripts/keypair-to-base58.js ~/deployer-raw.json

const fs = require("fs");
const bs58 = require("bs58");

const filePath = process.argv[2];
if (!filePath) {
  console.error("Usage: node scripts/keypair-to-base58.js <path-to-keypair.json>");
  process.exit(1);
}

const raw = JSON.parse(fs.readFileSync(filePath, "utf-8"));
const secretKey = Uint8Array.from(raw);

if (secretKey.length !== 64) {
  console.error(`Expected a 64-byte secret key, got ${secretKey.length} bytes.`);
  process.exit(1);
}

console.log(bs58.encode(secretKey));
