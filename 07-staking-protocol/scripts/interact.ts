import * as anchor from "@anchor-lang/core";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getAccount,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { StakingProtocol } from "../target/types/staking_protocol";
import idl from "../target/idl/staking_protocol.json";
import fs from "fs";
import os from "os";

// === Конфіг ===
const PROGRAM_ID = new anchor.web3.PublicKey(
  "8TDLJ18auzqhoQTFNsnijVSYBJNxxWqhdAvFw2shqsud"
);
const RPC = "https://api.devnet.solana.com";
const REWARD_RATE = new anchor.BN(1_000); // reward units / sec (на весь пул)

// === Утиліти ===
function explorer(kind: "tx" | "address", value: string): string {
  return `https://explorer.solana.com/${kind}/${value}?cluster=devnet`;
}

function loadKeypair(path: string): anchor.web3.Keypair {
  const raw = fs.readFileSync(path, "utf-8");
  const secretKey = Uint8Array.from(JSON.parse(raw));
  return anchor.web3.Keypair.fromSecretKey(secretKey);
}

async function main() {
  // === 1. Setup: connection, wallet, provider, program ===
  const connection = new anchor.web3.Connection(RPC, "confirmed");
  const wallet = loadKeypair(`${os.homedir()}/.config/solana/id.json`);

  console.log("Wallet:", wallet.publicKey.toBase58());
  const balance = await connection.getBalance(wallet.publicKey);
  console.log("Balance:", balance / anchor.web3.LAMPORTS_PER_SOL, "SOL\n");

  if (balance < 0.5 * anchor.web3.LAMPORTS_PER_SOL) {
    throw new Error("Замало SOL. Зроби `solana airdrop 2`.");
  }

  const provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(wallet),
    { commitment: "confirmed" }
  );
  const program = new anchor.Program<StakingProtocol>(
    idl as StakingProtocol,
    provider
  );

  // === 2. Створюємо stake_mint і reward_mint ===
  console.log("--- Creating mints ---");
  const stakeMint = await createMint(
    connection,
    wallet,
    wallet.publicKey,  // mint_authority
    null,              // freeze_authority
    6                  // decimals
  );
  console.log("stake_mint:", stakeMint.toBase58());
  console.log("    →", explorer("address", stakeMint.toBase58()));

  const rewardMint = await createMint(
    connection,
    wallet,
    wallet.publicKey,
    null,
    6
  );
  console.log("reward_mint:", rewardMint.toBase58());
  console.log("    →", explorer("address", rewardMint.toBase58()));

  // === 3. PDA адреси пулу ===
  const [pool] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), stakeMint.toBuffer()],
    PROGRAM_ID
  );
  const [vault] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), pool.toBuffer()],
    PROGRAM_ID
  );
  const [rewardVault] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("reward_vault"), pool.toBuffer()],
    PROGRAM_ID
  );
  const [userStake] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user"), pool.toBuffer(), wallet.publicKey.toBuffer()],
    PROGRAM_ID
  );

  console.log("\n--- PDAs ---");
  console.log("pool:        ", pool.toBase58());
  console.log("vault:       ", vault.toBase58());
  console.log("reward_vault:", rewardVault.toBase58());
  console.log("user_stake:  ", userStake.toBase58());

  // === 4. Initialize pool ===
  console.log("\n--- Initialize pool ---");
  const initTx = await program.methods
    .initializePool(REWARD_RATE)
    .accounts({
      authority: wallet.publicKey,
      stakeMint,
      rewardMint,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();
  console.log("TX:", initTx);
  console.log("    →", explorer("tx", initTx));

  // Перевіримо що пул реально записався
  const poolData = await program.account.stakingPool.fetch(pool);
  console.log("Pool state:");
  console.log("  authority:    ", poolData.authority.toBase58());
  console.log("  reward_rate:  ", poolData.rewardRate.toString());
  console.log("  total_staked: ", poolData.totalStaked.toString());

  // === 5. Готуємо юзера: ATA для stake_mint, мінтимо токени ===
  console.log("\n--- Prepare user stake balance ---");
  const userStakeAta = await getOrCreateAssociatedTokenAccount(
    connection,
    wallet,
    stakeMint,
    wallet.publicKey
  );
  console.log("user stake ATA:", userStakeAta.address.toBase58());

  await mintTo(
    connection,
    wallet,
    stakeMint,
    userStakeAta.address,
    wallet,
    1_000_000 // 1.0 з 6 decimals
  );
  console.log("Minted 1.0 stake tokens to user");

  // === 6. Поповнюємо reward_vault (адмін як mint_authority) ===
  console.log("\n--- Fund reward_vault ---");
  await mintTo(
    connection,
    wallet,
    rewardMint,
    rewardVault,
    wallet,
    1_000_000_000 // 1000.0 reward tokens
  );
  const rewardVaultAcc = await getAccount(connection, rewardVault);
  console.log("reward_vault balance:", rewardVaultAcc.amount.toString());

  // === 7. Stake 0.5 токенів ===
  console.log("\n--- Stake 0.5 ---");
  const stakeAmount = new anchor.BN(500_000);
  const stakeTx = await program.methods
    .stake(stakeAmount)
    .accountsPartial({
      user: wallet.publicKey,
      pool,
      userStake,
      userTokenAccount: userStakeAta.address,
      vault,
      stakeMint,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();
  console.log("TX:", stakeTx);
  console.log("    →", explorer("tx", stakeTx));

  // Перевіряємо стан пулу після stake
  const poolAfterStake = await program.account.stakingPool.fetch(pool);
  console.log("pool.total_staked:", poolAfterStake.totalStaked.toString());

  const userStakeData = await program.account.userStake.fetch(userStake);
  console.log("user.amount_staked:    ", userStakeData.amountStaked.toString());
  console.log("user.pending_rewards:  ", userStakeData.pendingRewards.toString());

  // === 8. Harvest — зразу після stake, очікуваний результат: 0 (часу не пройшло) ===
  console.log("\n--- Harvest (immediately after stake) ---");
  const userRewardAta = await getOrCreateAssociatedTokenAccount(
    connection,
    wallet,
    rewardMint,
    wallet.publicKey
  );
  console.log("user reward ATA:", userRewardAta.address.toBase58());

  const harvestTx = await program.methods
    .harvest()
    .accountsPartial({
      user: wallet.publicKey,
      pool,
      userStake,
      userRewardAccount: userRewardAta.address,
      rewardVault,
      rewardMint,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();
  console.log("TX:", harvestTx);
  console.log("    →", explorer("tx", harvestTx));

  const userRewardAfter = await getAccount(connection, userRewardAta.address);
  console.log(
    "user reward balance after harvest:",
    userRewardAfter.amount.toString(),
    "(0 очікувано — часу не минуло)"
  );

  // === Підсумок ===
  console.log("\n=== SUCCESS ===");
  console.log("Pool:        ", explorer("address", pool.toBase58()));
  console.log("Vault:       ", explorer("address", vault.toBase58()));
  console.log("Reward vault:", explorer("address", rewardVault.toBase58()));
  console.log("User stake:  ", explorer("address", userStake.toBase58()));
  console.log("\nЗачекай ~10 секунд і запусти ще раз --- harvest віддасть rewards.");
}

main().catch((e) => {
  console.error("ERROR:", e);
  process.exit(1);
});
