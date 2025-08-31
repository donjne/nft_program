use super::*;

#[event_cpi]
#[derive(Accounts)]
pub struct VerifyCollectionMint<'info> {
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub metadata: Account<'info, MetadataAccount>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    /// CHECK: This account is used for signing purposes only
    pub mint_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"nft", mint.key().as_ref()],
        bump = nft_info.bump,
    )]
    pub nft_info: Account<'info, NftInfo>,
    
    pub collection_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"collection", collection_mint.key().as_ref()],
        bump = collection_info.bump,
    )]
    pub collection_info: Account<'info, CollectionInfo>,
    
    #[account(mut)]
    pub collection_metadata: Account<'info, MetadataAccount>,
    
    pub collection_master_edition: Account<'info, MasterEditionAccount>,
    
    pub system_program: Program<'info, System>,
    
    #[account(address = INSTRUCTIONS_ID)]
    /// CHECK: Sysvar instruction account that is being checked with an address constraint
    pub sysvar_instruction: UncheckedAccount<'info>,
    
    pub token_metadata_program: Program<'info, Metadata>,
}

impl<'info> NftUtils for VerifyCollectionMint<'info> {}

impl<'info> VerifyCollectionMint<'info> {
    fn validate_collection_relationship(&self) -> Result<()> {
        if self.nft_info.collection_mint != self.collection_mint.key() {
            return Err(error!(NftError::InvalidCollectionMint));
        }

        if self.nft_info.verified {
            msg!("NFT is already verified in this collection");
        }
        
        Ok(())
    }
}

pub fn verify_collection(ctx: Context<VerifyCollectionMint>) -> Result<()> {
    ctx.accounts.validate_collection_relationship()?;
    
    let clock = Clock::get()?;

    let metadata = &ctx.accounts.metadata.to_account_info();
    let authority = &ctx.accounts.mint_authority.to_account_info();
    let collection_mint = &ctx.accounts.collection_mint.to_account_info();
    let collection_metadata = &ctx.accounts.collection_metadata.to_account_info();
    let collection_master_edition = &ctx.accounts.collection_master_edition.to_account_info();
    let system_program = &ctx.accounts.system_program.to_account_info();
    let sysvar_instructions = &ctx.accounts.sysvar_instruction.to_account_info();
    let spl_metadata_program = &ctx.accounts.token_metadata_program.to_account_info();

    let authority_bump = ctx.bumps.mint_authority;
    let seeds = &[&b"authority"[..], &[authority_bump]];
    let signer_seeds = &[&seeds[..]];

    let verify_collection = VerifyCollectionV1Cpi::new(
        spl_metadata_program,
        VerifyCollectionV1CpiAccounts {
            authority,
            delegate_record: None,
            metadata,
            collection_mint,
            collection_metadata: Some(collection_metadata),
            collection_master_edition: Some(collection_master_edition),
            system_program,
            sysvar_instructions,
        },
    );
    verify_collection
        .invoke_signed(signer_seeds)
        .map_err(|_| NftError::InvalidUri)?;
    
    msg!("Collection Verified!");

    let nft_info = &mut ctx.accounts.nft_info;
    if !nft_info.verified {
        nft_info.verified = true;

        let collection_info = &mut ctx.accounts.collection_info;
        collection_info.number_of_nfts = collection_info
            .number_of_nfts
            .checked_add(1)
            .ok_or(NftError::InvalidUri)?;
        
        msg!("Updated collection count to: {}", collection_info.number_of_nfts);
    }

    emit_cpi!(CollectionVerified {
        nft_mint: ctx.accounts.mint.key(),
        collection_mint: ctx.accounts.collection_mint.key(),
        authority: ctx.accounts.authority.key(),
        verified_at: clock.unix_timestamp,
    });
    
    Ok(())
}

