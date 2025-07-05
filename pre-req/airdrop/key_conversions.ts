import bs58 from 'bs58';
import promptSync from 'prompt-sync';

const prompt = promptSync();

// base58_to_wallet equivalent
function base58ToWallet() {
  const base58 = prompt('Enter base58 string:\n');
  try {
    const wallet = bs58.decode(base58);
    console.log('Decoded wallet (bytes):', Array.from(wallet));
  } catch (err) {
    console.error('Failed to decode base58:', err);
  }
}

// wallet_to_base58 equivalent
function walletToBase58() {
  const wallet = [
    34, 46, 55, 124, 141, 190, 24, 204, 134, 91, 70, 184, 161, 181, 44, 122,
    15, 172, 63, 62, 153, 150, 99, 255, 202, 89, 105, 77, 41, 89, 253, 130,
    27, 195, 134, 14, 66, 75, 24, 242, 7, 132, 234, 160, 203, 109, 195, 116,
    251, 144, 44, 28, 56, 231, 114, 50, 131, 185, 168, 138, 61, 35, 98, 78, 53
  ];
  const base58 = bs58.encode(Buffer.from(wallet));
  console.log('Base58 encoded wallet:', base58);
}

// Call the functions
walletToBase58();
base58ToWallet();
