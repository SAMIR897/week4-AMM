import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Amm } from "../target/types/amm";
import {
  createMint,
  createAccount,
  mintTo,
  getAccount,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";

describe("amm", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Amm as Program<Amm>;
  const wallet = provider.wallet as anchor.Wallet;

  let mintX: anchor.web3.PublicKey;
  let mintY: anchor.web3.PublicKey;
  let userX: anchor.web3.PublicKey;
  let userY: anchor.web3.PublicKey;
  
  const seed = new anchor.BN(Math.floor(Math.random() * 1000000));
  
  let config: anchor.web3.PublicKey;
  let mintLp: anchor.web3.PublicKey;
  let vaultX: anchor.web3.PublicKey;
  let vaultY: anchor.web3.PublicKey;
  let userLp: anchor.web3.PublicKey;
  let treasuryX: anchor.web3.PublicKey;
  let treasuryY: anchor.web3.PublicKey;

  const treasuryXKeypair = anchor.web3.Keypair.generate();
  const treasuryYKeypair = anchor.web3.Keypair.generate();

  before(async () => {
    mintX = await createMint(provider.connection, wallet.payer, wallet.publicKey, null, 6);
    mintY = await createMint(provider.connection, wallet.payer, wallet.publicKey, null, 6);

    userX = await createAccount(provider.connection, wallet.payer, mintX, wallet.publicKey);
    userY = await createAccount(provider.connection, wallet.payer, mintY, wallet.publicKey);
    treasuryX = await createAccount(provider.connection, wallet.payer, mintX, wallet.publicKey, treasuryXKeypair);
    treasuryY = await createAccount(provider.connection, wallet.payer, mintY, wallet.publicKey, treasuryYKeypair);

    await mintTo(provider.connection, wallet.payer, mintX, userX, wallet.payer, 1000000000);
    await mintTo(provider.connection, wallet.payer, mintY, userY, wallet.payer, 1000000000);

    [config] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("config"), seed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    [mintLp] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("lp"), config.toBuffer()],
      program.programId
    );
    [vaultX] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_x"), config.toBuffer()],
      program.programId
    );
    [vaultY] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_y"), config.toBuffer()],
      program.programId
    );
  });

  it("Initializes the AMM pool", async () => {
    const feeBps = 30;
    await program.methods
      .initialize(seed, feeBps)
      .accounts({
        initializer: wallet.publicKey,
        mintX,
        mintY,
        config,
        mintLp,
        vaultX,
        vaultY,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    userLp = await createAccount(provider.connection, wallet.payer, mintLp, wallet.publicKey);

    const configAccount = await program.account.config.fetch(config);
    assert.ok(configAccount.feeBasisPoints === feeBps);
    assert.ok(configAccount.authority.equals(wallet.publicKey));
  });

  it("Deposits liquidity", async () => {
    const amount = new anchor.BN(10000);
    const maxX = new anchor.BN(10000);
    const maxY = new anchor.BN(10000);

    await program.methods
      .deposit(amount, maxX, maxY)
      .accounts({
        user: wallet.publicKey,
        config,
        mintLp,
        mintX,
        mintY,
        userX,
        userY,
        userLp,
        vaultX,
        vaultY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const vaultXAccount = await getAccount(provider.connection, vaultX);
    const vaultYAccount = await getAccount(provider.connection, vaultY);
    const lpAccount = await getAccount(provider.connection, userLp);

    assert.equal(vaultXAccount.amount.toString(), "10000", "Vault X did not receive proper deposit");
    assert.equal(vaultYAccount.amount.toString(), "10000", "Vault Y did not receive proper deposit");
    assert.equal(lpAccount.amount.toString(), "10000", "User did not receive proper LP tokens");
  });

  it("Swaps tokens with proper fee math", async () => {
    const amountIn = new anchor.BN(10000);
    const minAmountOut = new anchor.BN(1);

    await program.methods
      .swap(true, amountIn, minAmountOut)
      .accounts({
        user: wallet.publicKey,
        config,
        mintX,
        mintY,
        userX,
        userY,
        vaultX,
        vaultY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const vaultXAccount = await getAccount(provider.connection, vaultX);
    const vaultYAccount = await getAccount(provider.connection, vaultY);
    const configAccount = await program.account.config.fetch(config);

    assert.equal(vaultXAccount.amount.toString(), "20000", "Vault X amount incorrect after swap");
    assert.ok(vaultYAccount.amount < 10000n, "Vault Y did not decrease after swap");
    assert.ok(configAccount.feeXPending.toNumber() > 0, "Fee was not collected into state");
  });

  it("Updates config parameters", async () => {
    await program.methods
      .updateConfig(50, null, null)
      .accounts({
        authority: wallet.publicKey,
        config,
      })
      .rpc();

    const configAccount = await program.account.config.fetch(config);
    assert.equal(configAccount.feeBasisPoints, 50, "Fee basis points did not update");
  });

  it("Collects accumulated fees to treasury", async () => {
    await program.methods
      .collectFees()
      .accounts({
        authority: wallet.publicKey,
        config,
        mintX,
        mintY,
        vaultX,
        vaultY,
        treasuryX,
        treasuryY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const configAccount = await program.account.config.fetch(config);
    const treasuryXAccount = await getAccount(provider.connection, treasuryX);

    assert.equal(configAccount.feeXPending.toNumber(), 0, "Pending fees should be reset to 0");
    assert.ok(treasuryXAccount.amount > 0n, "Treasury should have received collected fees");
  });

  it("Withdraws liquidity", async () => {
    const lpAmount = new anchor.BN(5000);
    const minX = new anchor.BN(1);
    const minY = new anchor.BN(1);

    await program.methods
      .withdraw(lpAmount, minX, minY)
      .accounts({
        user: wallet.publicKey,
        config,
        mintLp,
        mintX,
        mintY,
        userX,
        userY,
        userLp,
        vaultX,
        vaultY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const lpAccount = await getAccount(provider.connection, userLp);
    assert.equal(lpAccount.amount.toString(), "5000", "User LP tokens did not burn correctly");
  });
});
