import { Keypair } from "@solana/web3.js";

let kp = Keypair.generate();
console.log(`You've genereated a new solana wallet :${kp.publicKey.toBase58()}`);
console.log('To save your wallet, copy and paste your private key into a JSON file :');
console.log(`[${kp.secretKey}]`);