# NFT Program Testing Guide

This guide explains how to test the NFT program on both localnet and devnet environments.

## Prerequisites

Before testing, ensure you have:

- Anchor CLI installed (`anchor --version` should show 0.30.1 or 0.31.1)
- Solana CLI installed (`solana --version`)
- Node.js and npm/yarn installed
- A Solana wallet configured

## Project Structure

```bash
nft_program/
├── programs/
│   └── nft_program/
│       └── src/
│           ├── lib.rs
│           └── instructions/
├── tests/
│   └── nft_program.ts
├── target/
├── Anchor.toml
└── package.json
```

## Environment Setup

### Configure Solana CLI

```bash
# Set cluster (localnet/devnet)
solana config set --url http://127.0.0.1:8899  # for localnet
# or
solana config set --url https://api.devnet.solana.com  # for devnet

# Check current configuration
solana config get
```

### Install Dependencies

```bash
cd nft_program
npm install
# or
yarn install
```

## Testing on Localnet

Localnet testing allows you to test against cloned mainnet accounts, providing a more realistic environment.

### Step 1: Start Test Validator (Terminal 1)

```bash
solana-test-validator \
  --clone metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s \
  --clone PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT \
  --url https://api.mainnet-beta.solana.com \
  --reset
```

**What this command does:**

- `--clone metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s`: Clones Token Metadata Program
- `--clone PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT`: Clones Token Program
- `--url https://api.mainnet-beta.solana.com`: Source for cloning accounts
- `--reset`: Resets ledger state on startup

Wait for the message: `JSON RPC URL: http://127.0.0.1:8899`

### Step 2: Run Tests (Terminal 2)

```bash
cd nft_program

# Build the program first
anchor build

# Run tests (skips starting local validator since it's already running)
anchor test --skip-local-validator
```

### Expected Output

```bash
  nft-program
=== Creating Collection NFT ===
Collection Mint: [collection_mint_address]
Collection NFT created! TxID: [transaction_id]
    ✓ Create Collection NFT

=== Minting NFT ===
NFT Mint: [nft_mint_address]
NFT Minted! TxID: [transaction_id]
Updated Collection Count: 1
    ✓ Mint NFT

=== Verifying Collection ===
Collection Verified! TxID: [transaction_id]
NFT Verification Status: true
    ✓ Verify Collection

=== Reading Stored Data ===
Collection Data: { mint: '...', name: 'Test Collection', numberOfNfts: '1' }
NFT Data: { mint: '...', verified: true, ... }
    ✓ Read Collection and NFT Data

  4 passing
```

## Testing on Devnet

Devnet testing uses the actual Solana devnet, which requires SOL for transaction fees.

### Step 1: Configure for Devnet

```bash
# Set cluster to devnet
solana config set --url https://api.devnet.solana.com

# Check your wallet balance
solana balance

# If balance is low, request airdrop
solana airdrop 2
```

### Step 2: Update Anchor Configuration

In `Anchor.toml`, ensure devnet configuration:

```toml
[toolchain]

[features]
seeds = false
skip-lint = false

[programs.localnet]
nft_program = "YourProgramIdHere" eg. qYcgLKmGgHrREQcgFqVS7WqK35rh3kCXS6mG9T4SMjK

[programs.devnet]
nft_program = "YourProgramIdHere" eg. qYcgLKmGgHrREQcgFqVS7WqK35rh3kCXS6mG9T4SMjK

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "Devnet"  # Change this to "Devnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
```

### Step 3: Deploy and Test

```bash
cd nft_program

# Build the program
anchor build

# Deploy to devnet
anchor deploy

# Run tests on devnet
anchor test --provider.cluster devnet

# To inspect my devnet program id, go to the explorer and paste this address below:
# qYcgLKmGgHrREQcgFqVS7WqK35rh3kCXS6mG9T4SMjK
```

## Troubleshooting

### Common Issues

1. **"Program not found" error**

   ```bash
   # Redeploy the program
   anchor deploy 
   ```

2. **Insufficient SOL balance on devnet**

   ```bash
   solana airdrop 2 or go to https://faucet.solana.com
   ```

3. **RPC errors on localnet**
   - Restart the test validator
   - Check if all required programs are cloned

4. **Build failures**

   ```bash
   # Clean and rebuild
   anchor clean
   anchor build
   ```

### Viewing Transaction Details

```bash
# View transaction on explorer (replace with actual transaction signature)
solana confirm [TRANSACTION_SIGNATURE] --verbose
```

For devnet transactions, you can also view them on:

- <https://explorer.solana.com/?cluster=devnet>
