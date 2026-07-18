// Uploads assets/image.png + a Metaplex-standard metadata JSON to IPFS via
// Pinata's REST API (no SDK needed - just two fetch calls with a JWT).
// Prints the metadata URI to pass into `mint_nft` (see scripts/mint-nft.ts).
//
// Why IPFS at all? Solana accounts are expensive to rent per byte, so NFTs
// never store the image or a big JSON blob on-chain. The on-chain metadata
// account just stores a `uri` (a string, capped at 200 chars in this
// program) pointing at where the real JSON lives - IPFS is the de facto
// standard because it's content-addressed (the CID *is* a hash of the
// content, so nobody, including you, can quietly swap the file later).
import "dotenv/config";
import fs from "fs";
import path from "path";
// @ts-ignore - no types package for this, and we only use the constructor
import FormData from "form-data";

const PINATA_API = "https://api.pinata.cloud";

async function pinataRequest(endpoint: string, body: FormData | object) {
  const jwt = process.env.PINATA_JWT;
  if (!jwt) throw new Error("PINATA_JWT missing from .env - see README for how to get one");

  const isForm = body instanceof FormData;
  const res = await fetch(`${PINATA_API}${endpoint}`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${jwt}`,
      ...(isForm ? {} : { "Content-Type": "application/json" }),
    },
    // @ts-ignore - node-fetch/undici accept FormData's stream fine at runtime
    body: isForm ? (body as any) : JSON.stringify(body),
  });

  if (!res.ok) {
    throw new Error(`Pinata ${endpoint} failed: ${res.status} ${await res.text()}`);
  }
  return res.json();
}

async function main() {
  const name = process.argv[2] || "My First Solana NFT";
  const symbol = process.argv[3] || "LECT3";
  const description = process.argv[4] || "Minted in lecture 3 - Anchor + Metaplex + IPFS";

  const imagePath = path.join(__dirname, "..", "assets", "image.png");
  if (!fs.existsSync(imagePath)) {
    throw new Error(
      `No image at ${imagePath}. Drop a .png there first (any image works for class).`
    );
  }

  console.log("Uploading image to IPFS...");
  const imageForm = new FormData();
  imageForm.append("file", fs.createReadStream(imagePath));
  const imageResult: any = await pinataRequest("/pinning/pinFileToIPFS", imageForm);
  const imageCid = imageResult.IpfsHash;
  const imageUri = `ipfs://${imageCid}`;
  console.log(`Image pinned: ${imageUri}`);
  console.log(`  Gateway preview: https://gateway.pinata.cloud/ipfs/${imageCid}`);

  const metadata = {
    name,
    symbol,
    description,
    image: imageUri,
    attributes: [{ trait_type: "Cohort", value: "Solana + Rust Lecture 3" }],
    properties: {
      files: [{ uri: imageUri, type: "image/png" }],
      category: "image",
    },
  };

  console.log("Uploading metadata JSON to IPFS...");
  const metadataResult: any = await pinataRequest("/pinning/pinJSONToIPFS", metadata);
  const metadataCid = metadataResult.IpfsHash;
  const metadataUri = `ipfs://${metadataCid}`;

  console.log(`\nMetadata pinned: ${metadataUri}`);
  console.log(`  Gateway preview: https://gateway.pinata.cloud/ipfs/${metadataCid}`);

  const outPath = path.join(__dirname, "..", "pending-mint.json");
  fs.writeFileSync(
    outPath,
    JSON.stringify({ name, symbol, uri: metadataUri }, null, 2)
  );
  console.log(`\nSaved to ${outPath} - now run: yarn mint`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
