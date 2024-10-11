// No imports needed: web3, anchor, pg and more are globally available

describe("Stablecoin Protocol Tests", () => {
  let userAccountKp;
  let userStablecoinAccount;
  let stablecoinMint;
  let governanceAccountKp;

  before(async () => {
    // Set up keypairs and initial accounts before running tests
    userAccountKp = new web3.Keypair();
    governanceAccountKp = new web3.Keypair();
    stablecoinMint = new web3.Keypair();

    // Create a token account for the user
    userStablecoinAccount = await pg.createTokenAccount(stablecoinMint.publicKey, pg.wallet.publicKey);
  });

  it("initialize", async () => {
    // Collateral ratio to initialize with
    const collateralRatio = new BN(150); // Example value for collateral ratio

    // Send transaction to initialize governance
    const txHash = await pg.program.methods
      .initialize(collateralRatio)
      .accounts({
        governance: governanceAccountKp.publicKey,
        payer: pg.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([governanceAccountKp])
      .rpc();

    console.log(`Initialize TX Hash: ${txHash}`);

    // Confirm transaction
    await pg.connection.confirmTransaction(txHash);

    // Fetch the governance account data
    const governanceAccount = await pg.program.account.governance.fetch(governanceAccountKp.publicKey);
    console.log("On-chain governance data:", governanceAccount.collateralRatio.toString());

    // Check if the collateral ratio matches the expected value
    assert(collateralRatio.eq(new BN(governanceAccount.collateralRatio)));
  });

  it("mint_stablecoin", async () => {
    // Amount to mint
    const mintAmount = new BN(1000);
    const currentPrice = new BN(110); // Example current price of the stablecoin

    // Send transaction to mint stablecoin
    const txHash = await pg.program.methods
      .mintStablecoin(mintAmount, currentPrice)
      .accounts({
        userAccount: userAccountKp.publicKey,
        userStablecoinAccount: userStablecoinAccount,
        stablecoinMint: stablecoinMint.publicKey,
        treasuryAccount: pg.wallet.publicKey, // Assuming the treasury is controlled by the payer
        tokenProgram: web3.TokenProgram.programId,
        payer: pg.wallet.publicKey,
      })
      .signers([userAccountKp])
      .rpc();

    console.log(`Mint Stablecoin TX Hash: ${txHash}`);

    // Confirm transaction
    await pg.connection.confirmTransaction(txHash);

    // Fetch the user account and check the stablecoin balance
    const userAccount = await pg.program.account.userAccount.fetch(userAccountKp.publicKey);
    console.log("User stablecoin balance:", userAccount.stablecoinBalance.toString());

    // Ensure the minted amount is reflected in the user's account
    assert(mintAmount.eq(new BN(userAccount.stablecoinBalance)));
  });

  it("partial_liquidate", async () => {
    // Amount to liquidate
    const liquidationAmount = new BN(500);

    // Send transaction to partially liquidate user's position
    const txHash = await pg.program.methods
      .partialLiquidate(liquidationAmount)
      .accounts({
        userAccount: userAccountKp.publicKey,
        liquidatorCollateralAccount: userStablecoinAccount,
        tokenProgram: web3.TokenProgram.programId,
        payer: pg.wallet.publicKey,
      })
      .signers([userAccountKp])
      .rpc();

    console.log(`Partial Liquidate TX Hash: ${txHash}`);

    // Confirm transaction
    await pg.connection.confirmTransaction(txHash);

    // Fetch the user account and check the new stablecoin balance
    const userAccount = await pg.program.account.userAccount.fetch(userAccountKp.publicKey);
    console.log("User stablecoin balance after liquidation:", userAccount.stablecoinBalance.toString());

    // Ensure the stablecoin balance reflects the liquidation
    const expectedBalance = new BN(1000).sub(liquidationAmount); // Original balance minus liquidation
    assert(expectedBalance.eq(new BN(userAccount.stablecoinBalance)));
  });
});
