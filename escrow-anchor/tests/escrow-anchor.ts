import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EscrowAnchor } from "../target/types/escrow_anchor";
import {
  createMint,
  mintTo,
  createAccount,
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountIdempotentInstruction,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { expect } from "chai";

async function logAddressBalance(
  pubKey: anchor.web3.PublicKey,
  logString: String,
  provider: anchor.AnchorProvider
) {
  const lamports = await provider.connection.getBalance(pubKey);
  console.log(`${logString} ${pubKey} : ${lamports}`);
  return lamports;
}

describe("escrow-anchor", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.escrowAnchor as Program<EscrowAnchor>;
  const tokenMaker = provider.wallet;
  let ataMakerMintA: undefined | anchor.web3.PublicKey; // Can change its type to store ataMakerMintA details
  let ataTakerMintB: undefined | anchor.web3.PublicKey; // Can change its type to store ataTakerMintB details

  const [maker, taker, mintA, mintB] = Array.from({ length: 4 }, () => {
    return anchor.web3.Keypair.generate();
  });

  let escrowTokenController: anchor.web3.PublicKey;
  let escrowTokenAccA: anchor.web3.PublicKey;
  let escrowTokenAccB: anchor.web3.PublicKey;
  let userADetailsPda: anchor.web3.PublicKey;
  let userBDetailsPda: anchor.web3.PublicKey;
  let dealDetailsPda: anchor.web3.PublicKey;

  // Purpose of this block is to
  // Airdrop SOL to maker
  // Create and initialize two new token mints called mint_a, mint_b which are token_2022 accounts with mint_authority given to tokenMaker
  // Create two ATA for maker and taker for mint_a and mint_b respectively
  // Mint some amount of mint_a token to maker and mint_b token to taker
  before(async () => {
    // Airdrop SOL to maker
    [maker,taker].forEach(async user => {
        const airDropTx = await provider.connection.requestAirdrop(
        user.publicKey,
        100 * anchor.web3.LAMPORTS_PER_SOL
      );
      const latestBlock = await provider.connection.getLatestBlockhash();
      await provider.connection.confirmTransaction(
        {
          blockhash: latestBlock.blockhash,
          lastValidBlockHeight: latestBlock.lastValidBlockHeight,
          signature: airDropTx
        },
        "confirmed",
      );
      await logAddressBalance(user.publicKey, "User SOL Balance:", provider);
    })

    await createMint(
      provider.connection,
      tokenMaker.payer, // Payer for the transaction
      tokenMaker.publicKey, // Mint Authority
      tokenMaker.publicKey, // Freeze Authority
      3, // Decimals
      mintA, // Keypair for the new mint account
      undefined, // Confirm options
      TOKEN_2022_PROGRAM_ID // Use TOKEN_2022_PROGRAM_ID
    );

    // Create and initialize mintB (token-2022)
    await createMint(
      provider.connection,
      tokenMaker.payer,
      tokenMaker.publicKey,
      tokenMaker.publicKey,
      2,
      mintB,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    console.log("Mint A created:", mintA.publicKey.toBase58());
    console.log("Mint B created:", mintB.publicKey.toBase58());

    // Create ATAs for maker (mintA) and taker (mintB)
    ataMakerMintA = await createAccount(
      provider.connection,
      tokenMaker.payer, // Payer for the transaction
      mintA.publicKey,
      maker.publicKey,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    ataTakerMintB = await createAccount(
      provider.connection,
      tokenMaker.payer,
      mintB.publicKey,
      taker.publicKey,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    console.log("Maker's ATA for Mint A:", ataMakerMintA.toBase58());
    console.log("Taker's ATA for Mint B:", ataTakerMintB.toBase58());

    await mintTo(
      provider.connection,
      tokenMaker.payer, // Payer for the transaction
      mintA.publicKey, // Mint account
      ataMakerMintA, // Destination ATA
      tokenMaker.publicKey, // Mint authority
      3000,
      [], // Signers if mint authority is not provider.wallet
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    await mintTo(
      provider.connection,
      tokenMaker.payer,
      mintB.publicKey,
      ataTakerMintB,
      tokenMaker.publicKey,
      2000,
      [],
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    const makerAmount = await provider.connection.getTokenAccountBalance(
      ataMakerMintA,
      "confirmed"
    );
    const takerAmount = await provider.connection.getTokenAccountBalance(
      ataTakerMintB,
      "confirmed"
    );
    console.log("Checking ATA Maker : ", makerAmount.value.uiAmountString);
    console.log("Checking ATA Taker : ", takerAmount.value.uiAmountString);

    // Calculate PDAs once to use in tests
    [dealDetailsPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("deal"), maker.publicKey.toBuffer()],
      program.programId
    );

    [escrowTokenController] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("controller"), maker.publicKey.toBuffer()],
      program.programId
    );

    [escrowTokenAccA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("token"), maker.publicKey.toBuffer()],
      program.programId
    );

    [escrowTokenAccB] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("token"), taker.publicKey.toBuffer()],
      program.programId
    );

    [userADetailsPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user_details"), maker.publicKey.toBuffer()],
      program.programId
    );

    [userBDetailsPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user_details"), taker.publicKey.toBuffer()],
      program.programId
    );

    console.log("Calculated PDAs:");
    console.log("dealDetailsPda:", dealDetailsPda.toBase58());
    console.log("escrowTokenController:", escrowTokenController.toBase58());
    console.log("escrowTokenAccA:", escrowTokenAccA.toBase58());
    console.log("escrowTokenAccB:", escrowTokenAccB.toBase58());
    console.log("userADetailsPda:", userADetailsPda.toBase58());
    console.log("userBDetailsPda:", userBDetailsPda.toBase58());
  });

  it("Create escrow", async () => {
    const maker_amt = new anchor.BN(1000);
    const taker_amt = new anchor.BN(1500);
    // Log initial balances for verification
    const initialMakerATABalance =
      await provider.connection.getTokenAccountBalance(
        ataMakerMintA!,
        "confirmed"
      );
    console.log(
      "Initial Maker Mint A ATA Balance:",
      initialMakerATABalance.value.amount
    );

    const tx = await program.methods
      .create(maker_amt, taker_amt)
      .accounts({
        maker: maker.publicKey,
        taker: taker.publicKey,
        mintA: mintA.publicKey,
        mintB: mintB.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID, // Use the correct program ID
      })
      .signers([maker]) // The 'maker' is the signer for this instruction
      .rpc();

    console.log("Your transaction signature", tx);

    // 1. Verify DealDetails account was created and data is correct
    const dealDetailsAccount = await program.account.dealDetails.fetch(
      dealDetailsPda
    );
    expect(dealDetailsAccount.maker.equals(maker.publicKey)).to.be.true;
    expect(dealDetailsAccount.taker.equals(taker.publicKey)).to.be.true;
    expect(dealDetailsAccount.isFullfilled).to.be.false;

    // 2. Verify UserADetails account was created and data is correct
    const userADetailsAccount = await program.account.userEscrowDetails.fetch(
      userADetailsPda
    );
    expect(userADetailsAccount.mintAmt.eq(maker_amt)).to.be.true;
    expect(userADetailsAccount.mint.equals(mintA.publicKey)).to.be.true;

    // 3. Verify UserBDetails account was created and data is correct
    const userBDetailsAccount = await program.account.userEscrowDetails.fetch(
      userBDetailsPda
    );
    expect(userBDetailsAccount.mintAmt.eq(taker_amt)).to.be.true;
    expect(userBDetailsAccount.mint.equals(mintB.publicKey)).to.be.true;

    // 4. Verify tokens were transferred from maker's ATA to escrowTokenAccA
    const finalMakerATABalance =
      await provider.connection.getTokenAccountBalance(
        ataMakerMintA!,
        "confirmed"
      );
    const escrowATABalance = await provider.connection.getTokenAccountBalance(
      escrowTokenAccA,
      "confirmed"
    );

    console.log(
      "Final Maker Mint A ATA Balance:",
      finalMakerATABalance.value.amount
    );
    console.log(
      "Escrow Mint A Token Account Balance:",
      escrowATABalance.value.amount
    );

    expect(escrowATABalance.value.uiAmount).eq(1);
  });

  it("Deposit amount to existing deal", async () => {
    await program.methods
      .deposit()
      .accounts({
        maker: maker.publicKey,
        taker: taker.publicKey,
        mint: mintB.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([taker])
      .rpc();

    // const takerATAamount = await provider.connection.getTokenAccountBalance(escrowTokenAccB, 'confirmed');
    // expect(takerATAamount.value.uiAmount).eq(15);

    // Check if deal_details.is_fullfilled is true
    const dealDetails = await program.account.dealDetails.fetch(dealDetailsPda);
    expect(dealDetails.isFullfilled).eq(true);
  });

  it("Withdraw funds from escrow accounts", async () => {
    // create a new token account for the maker and taker
    // hit the withdraw instruction with maker or taker ids
    // check if escrow account is empty and funds are transferred to maker or taker account

    // Create token account of mintB for maker (we are assuming here that user has already made their token account)
    const makerMintBata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      maker,
      mintB.publicKey,
      maker.publicKey,
      undefined,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    // Checking makerMintBAta amount before
    let makerMintBataAmount = await provider.connection.getTokenAccountBalance(makerMintBata.address);
    expect(makerMintBataAmount.value.uiAmount).eq(0);

    const makerWithdrawIx = await program.methods
      .withdraw()
      .accounts({
        maker: maker.publicKey,
        taker: taker.publicKey,
        signer: maker.publicKey,
        userTokenAcc: makerMintBata.address,
        mintA: mintA.publicKey,
        mintB: mintB.publicKey,
        mintExchange: mintB.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([maker])
      .rpc();

    console.log("Maker successfully withdrawn : ", makerWithdrawIx);

    makerMintBataAmount = await provider.connection.getTokenAccountBalance(makerMintBata.address);
    expect(makerMintBataAmount.value.uiAmount).eq(15);

    const takerMintAata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      taker,
      mintA.publicKey,
      taker.publicKey,
      undefined,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );


    let takerMintAataAmount = await provider.connection.getTokenAccountBalance(takerMintAata.address);

    const takerWithdrawIx = await program.methods
      .withdraw()
      .accounts({
        maker: maker.publicKey,
        taker: taker.publicKey,
        signer: taker.publicKey,
        userTokenAcc: takerMintAata.address,
        mintA: mintA.publicKey,
        mintB: mintB.publicKey,
        mintExchange: mintA.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([taker])
      .rpc();
    
    console.log("Taker successfully withdrawn : ", takerWithdrawIx);

    takerMintAataAmount = await provider.connection.getTokenAccountBalance(takerMintAata.address);
    expect(takerMintAataAmount.value.uiAmount).eq(1);
    
  });

  it("Close all accounts and transfer funds to maker ", async () => {
    let makerAccountBalance = await provider.connection.getBalance(maker.publicKey);
    console.log("Makers balance before : ", makerAccountBalance);

    const tx = await program.methods.close().accounts({
      maker: maker.publicKey,
      taker: taker.publicKey,
      tokenProgram: TOKEN_2022_PROGRAM_ID
    }).signers([maker]).rpc();

    console.log("All accounds closed succesfully : ", tx);

    makerAccountBalance = await provider.connection.getBalance(maker.publicKey);
    console.log("Makers balance after : ", makerAccountBalance);

  })
});
