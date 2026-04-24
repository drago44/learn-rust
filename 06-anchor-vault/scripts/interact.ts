import * as anchor from "@anchor-lang/core";
import { AnchorVault } from "../target/types/anchor_vault";
import idl from "../target/idl/anchor_vault.json";
import fs from "fs";
import os from "os";

const PROGRAM_ID = new anchor.web3.PublicKey(
  "6doU3yt22cojkmVxdWeNTWSxYiRDXEjQueBJkRf1dbQ5"
);

async function main() {
  // підключення до devnet
  const connection = new anchor.web3.Connection(
    "https://api.devnet.solana.com",
    "confirmed"
  );

  // завантажуємо локальний keypair (~/.config/solana/id.json)
  const keypairPath = `${os.homedir()}/.config/solana/id.json`;
  const raw = fs.readFileSync(keypairPath, "utf-8");
  const secretKey = Uint8Array.from(JSON.parse(raw));
  const wallet = anchor.web3.Keypair.fromSecretKey(secretKey);

  console.log("Wallet:", wallet.publicKey.toBase58());
  console.log(
    "Balance:",
    (await connection.getBalance(wallet.publicKey)) / anchor.web3.LAMPORTS_PER_SOL,
    "SOL"
  );

  const provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(wallet),
    { commitment: "confirmed" }
  );

  const program = new anchor.Program<AnchorVault>(idl as AnchorVault, provider);

  // PDA адреси
  const [vaultState] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("state"), wallet.publicKey.toBuffer()],
    PROGRAM_ID
  );
  const [vault] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), wallet.publicKey.toBuffer()],
    PROGRAM_ID
  );

  console.log("\nVault State PDA:", vaultState.toBase58());
  console.log("Vault PDA:", vault.toBase58());

  // initialize
  console.log("\n--- Initialize ---");
  try {
    const tx = await program.methods
      .initialize()
      .accounts({ owner: wallet.publicKey })
      .rpc();
    console.log("TX:", tx);
    console.log("Vault initialized!");
  } catch (e: any) {
    console.log("Already initialized:", e.message);
  }

  // deposit 0.1 SOL
  console.log("\n--- Deposit 0.1 SOL ---");
  const depositAmount = new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL);
  const depositTx = await program.methods
    .deposit(depositAmount)
    .accounts({ owner: wallet.publicKey })
    .rpc();
  console.log("TX:", depositTx);

  const vaultBalance = await connection.getBalance(vault);
  console.log("Vault balance:", vaultBalance / anchor.web3.LAMPORTS_PER_SOL, "SOL");

  // withdraw 0.05 SOL
  console.log("\n--- Withdraw 0.05 SOL ---");
  const withdrawAmount = new anchor.BN(0.05 * anchor.web3.LAMPORTS_PER_SOL);
  const withdrawTx = await program.methods
    .withdraw(withdrawAmount)
    .accounts({ owner: wallet.publicKey })
    .rpc();
  console.log("TX:", withdrawTx);

  const vaultBalanceAfter = await connection.getBalance(vault);
  console.log(
    "Vault balance after withdraw:",
    vaultBalanceAfter / anchor.web3.LAMPORTS_PER_SOL,
    "SOL"
  );

  // вивести залишок
  if (vaultBalanceAfter > 0) {
    console.log("\n--- Withdraw rest ---");
    const restTx = await program.methods
      .withdraw(new anchor.BN(vaultBalanceAfter))
      .accounts({ owner: wallet.publicKey })
      .rpc();
    console.log("TX:", restTx);
    console.log("Vault balance final:", await connection.getBalance(vault), "lamports");
  }
}

main().catch(console.error);
