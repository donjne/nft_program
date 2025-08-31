import * as anchor from '@coral-xyz/anchor';
import type { Program } from '@coral-xyz/anchor';
import type NodeWallet from '@coral-xyz/anchor/dist/cjs/nodewallet';
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync } from '@solana/spl-token';
import { Keypair, SystemProgram } from '@solana/web3.js';
import type { NftProgram } from '../target/types/nft_program';

describe('nft-program', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = provider.wallet as NodeWallet;
  const program = anchor.workspace.NftProgram as Program<NftProgram>;

  const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');

  const mintAuthority = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('authority')], 
    program.programId
  )[0];

  const collectionKeypair = Keypair.generate();
  const collectionMint = collectionKeypair.publicKey;
  const mintKeypair = Keypair.generate();
  const mint = mintKeypair.publicKey;

  const getMetadata = (mint: anchor.web3.PublicKey): anchor.web3.PublicKey => {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('metadata'), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      TOKEN_METADATA_PROGRAM_ID,
    )[0];
  };

  const getMasterEdition = (mint: anchor.web3.PublicKey): anchor.web3.PublicKey => {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('metadata'), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from('edition')],
      TOKEN_METADATA_PROGRAM_ID,
    )[0];
  };

  const getCollectionInfo = (mint: anchor.web3.PublicKey): anchor.web3.PublicKey => {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('collection'), mint.toBuffer()],
      program.programId,
    )[0];
  };

  const getNftInfo = (mint: anchor.web3.PublicKey): anchor.web3.PublicKey => {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('nft'), mint.toBuffer()],
      program.programId,
    )[0];
  };

  const collectionData = {
    name: "Test Collection",
    symbol: "TEST",
    uri: "https://example.com/collection.json",
    sellerFeeBasisPoints: 500, // 5%
    creators: [
      {
        address: wallet.publicKey,
        verified: false,
        share: 100,
      }
    ],
  };

  // Sample NFT data
  const nftData = {
    name: "Test NFT #1",
    symbol: "TNFT",
    uri: "https://example.com/nft1.json",
    sellerFeeBasisPoints: 250, // 2.5%
    creators: [
      {
        address: wallet.publicKey,
        verified: false,
        share: 100,
      }
    ],
  };

  it('Create Collection NFT', async () => {
    console.log('\n=== Creating Collection NFT ===');
    console.log('Collection Mint:', collectionMint.toBase58());

    const metadata = getMetadata(collectionMint);
    const masterEdition = getMasterEdition(collectionMint);
    const collectionInfo = getCollectionInfo(collectionMint);
    const destination = getAssociatedTokenAddressSync(collectionMint, wallet.publicKey);

    console.log('Collection Metadata:', metadata.toBase58());
    console.log('Master Edition:', masterEdition.toBase58());
    console.log('Collection Info PDA:', collectionInfo.toBase58());
    console.log('Destination ATA:', destination.toBase58());

    const tx = await program.methods
      .createCollectionInstruction(collectionData)
      .accountsPartial({
        user: wallet.publicKey,
        mint: collectionMint,
        mintAuthority,
        collectionInfo,
        metadata,
        masterEdition,
        destination,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      })
      .signers([collectionKeypair])
      .rpc({
        skipPreflight: true,
      });

    console.log('Collection NFT created! TxID:', tx);

    const collectionInfoAccount = await program.account.collectionInfo.fetch(collectionInfo);
    console.log('Collection Info:', {
      mint: collectionInfoAccount.mint.toBase58(),
      creator: collectionInfoAccount.creator.toBase58(),
      numberOfNfts: collectionInfoAccount.numberOfNfts.toString(),
    });
  });

  it('Mint NFT', async () => {
    console.log('\n=== Minting NFT ===');
    console.log('NFT Mint:', mint.toBase58());

    const metadata = getMetadata(mint);
    const masterEdition = getMasterEdition(mint);
    const nftInfo = getNftInfo(mint);
    const collectionInfo = getCollectionInfo(collectionMint);
    const destination = getAssociatedTokenAddressSync(mint, wallet.publicKey);

    console.log('NFT Metadata:', metadata.toBase58());
    console.log('Master Edition:', masterEdition.toBase58());
    console.log('NFT Info PDA:', nftInfo.toBase58());
    console.log('Collection Info PDA:', collectionInfo.toBase58());
    console.log('Destination ATA:', destination.toBase58());

    const tx = await program.methods
      .mintNftInstruction(nftData)
      .accountsPartial({
        owner: wallet.publicKey,
        mint: mint,
        destination,
        mintAuthority,
        nftInfo,
        metadata,
        masterEdition,
        collectionMint,
        //@ts-ignore
        collectionInfo,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      })
      .signers([mintKeypair])
      .rpc({
        skipPreflight: true,
      });

    console.log('NFT Minted! TxID:', tx);

    const nftInfoAccount = await program.account.nftInfo.fetch(nftInfo);
    console.log('NFT Info:', {
      mint: nftInfoAccount.mint.toBase58(),
      collectionMint: nftInfoAccount.collectionMint.toBase58(),
      owner: nftInfoAccount.owner.toBase58(),
      verified: nftInfoAccount.verified,
    });

    const collectionInfoAccount = await program.account.collectionInfo.fetch(collectionInfo);
    console.log('Updated Collection Count:', collectionInfoAccount.numberOfNfts.toString());
  });

  it('Verify Collection', async () => {
    console.log('\n=== Verifying Collection ===');

    const mintMetadata = getMetadata(mint);
    const nftInfo = getNftInfo(mint);
    const collectionInfo = getCollectionInfo(collectionMint);
    const collectionMetadata = getMetadata(collectionMint);
    const collectionMasterEdition = getMasterEdition(collectionMint);

    console.log('NFT Metadata:', mintMetadata.toBase58());
    console.log('NFT Info PDA:', nftInfo.toBase58());
    console.log('Collection Info PDA:', collectionInfo.toBase58());
    console.log('Collection Metadata:', collectionMetadata.toBase58());
    console.log('Collection Master Edition:', collectionMasterEdition.toBase58());

    const tx = await program.methods
      .verifyCollectionInstruction()
      .accountsPartial({
        authority: wallet.publicKey,
        metadata: mintMetadata,
        mint: mint,
        mintAuthority,
        nftInfo,
        collectionMint,
        collectionInfo,
        collectionMetadata,
        collectionMasterEdition,
        systemProgram: SystemProgram.programId,
        sysvarInstruction: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      })
      .rpc({
        skipPreflight: true,
      });

    console.log('Collection Verified! TxID:', tx);

    const nftInfoAccount = await program.account.nftInfo.fetch(nftInfo);
    console.log('NFT Verification Status:', nftInfoAccount.verified);
  });

  it('Read Collection and NFT Data', async () => {
    console.log('\n=== Reading Stored Data ===');

    const collectionInfo = getCollectionInfo(collectionMint);
    const nftInfo = getNftInfo(mint);

    const collectionInfoAccount = await program.account.collectionInfo.fetch(collectionInfo);

    const collectionName = new TextDecoder().decode(
      new Uint8Array(collectionInfoAccount.name.filter(b => b !== 0))
    );
    const collectionSymbol = new TextDecoder().decode(
      new Uint8Array(collectionInfoAccount.symbol.filter(b => b !== 0))
    );
    const collectionUri = new TextDecoder().decode(
      new Uint8Array(collectionInfoAccount.uri.filter(b => b !== 0))
    );

    console.log('Collection Data:', {
      mint: collectionInfoAccount.mint.toBase58(),
      name: collectionName,
      symbol: collectionSymbol,
      uri: collectionUri,
      creator: collectionInfoAccount.creator.toBase58(),
      numberOfNfts: collectionInfoAccount.numberOfNfts.toString(),
      createdAt: new Date(collectionInfoAccount.createdAt.toNumber() * 1000).toISOString(),
    });

    const nftInfoAccount = await program.account.nftInfo.fetch(nftInfo);
    
    const nftName = new TextDecoder().decode(
      new Uint8Array(nftInfoAccount.name.filter(b => b !== 0))
    );
    const nftSymbol = new TextDecoder().decode(
      new Uint8Array(nftInfoAccount.symbol.filter(b => b !== 0))
    );
    const nftUri = new TextDecoder().decode(
      new Uint8Array(nftInfoAccount.uri.filter(b => b !== 0))
    );

    console.log('NFT Data:', {
      mint: nftInfoAccount.mint.toBase58(),
      collectionMint: nftInfoAccount.collectionMint.toBase58(),
      name: nftName,
      symbol: nftSymbol,
      uri: nftUri,
      owner: nftInfoAccount.owner.toBase58(),
      verified: nftInfoAccount.verified,
      mintedAt: new Date(nftInfoAccount.mintedAt.toNumber() * 1000).toISOString(),
    });
  });
});
