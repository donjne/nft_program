use anchor_lang::prelude::*;
pub use anchor_lang::solana_program::sysvar::instructions::ID as INSTRUCTIONS_ID;
use anchor_spl::{
    token::{TokenAccount, Token, Mint, MintTo, mint_to}, 
    metadata::{
        MasterEditionAccount, 
        MetadataAccount,
        Metadata
    },
    associated_token::AssociatedToken,
    metadata::mpl_token_metadata::{
        self,
        instructions::{
            CreateMasterEditionV3Cpi, 
            CreateMasterEditionV3CpiAccounts, 
            CreateMasterEditionV3InstructionArgs, 
            CreateMetadataAccountV3Cpi, 
            CreateMetadataAccountV3CpiAccounts, 
            CreateMetadataAccountV3InstructionArgs,
            VerifyCollectionV1Cpi,
            VerifyCollectionV1CpiAccounts,
        }, 
        types::{
            Collection,  
            Creator, 
            DataV2
        }
    }
};

pub mod errors;
pub mod instructions;
pub mod states;
pub mod events;

pub use errors::NftError;
pub use instructions::*;
pub use states::*;
pub use events::*;

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "NFT Program",
    project_url: "https://github.com/donjne/nft_program",
    contacts: "email:davidjrn247@gmail.com",
    policy: "Build quality software, test thoroughly, and deploy with confidence.",
    source_code: "https://github.com/donjne/nft_program",
    source_release: "v1.0.0",
    auditors: "David J.N",
    acknowledgements: "Superteam Earn & Codigo"
}

declare_id!("qYcgLKmGgHrREQcgFqVS7WqK35rh3kCXS6mG9T4SMjK");

#[program]
pub mod nft_program {
    use super::*;

    pub fn create_collection_instruction(
        ctx: Context<CreateCollection>,
        collection_data: CollectionData,
    ) -> Result<()> {
        create_collection(ctx, collection_data)
    }
    
    pub fn mint_nft_instruction(
        ctx: Context<MintNFT>,
        nft_data: NftData,
    ) -> Result<()> {
        mint_nft(ctx, nft_data)
    }

    pub fn verify_collection_instruction(
        ctx: Context<VerifyCollectionMint>,
    ) -> Result<()> {
        verify_collection(ctx)
    }
}
