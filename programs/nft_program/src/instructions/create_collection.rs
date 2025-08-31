use super::*;
use crate::instructions::shared::validation::ValidatableData;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CollectionData {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Vec<CreatorData>,
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(collection_data: CollectionData)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        init,
        payer = user,
        mint::decimals = 0,
        mint::authority = mint_authority,
        mint::freeze_authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    /// CHECK: This account is used for signing purposes only
    pub mint_authority: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = user,
        space = 8 + CollectionInfo::INIT_SPACE,
        seeds = [b"collection", mint.key().as_ref()],
        bump,
    )]
    pub collection_info: Account<'info, CollectionInfo>,
    
    #[account(mut)]
    /// CHECK: This account will be initialized by the metaplex program
    pub metadata: UncheckedAccount<'info>,
    
    #[account(mut)]
    /// CHECK: This account will be initialized by the metaplex program
    pub master_edition: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub destination: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
}

impl validation::ValidatableData for CollectionData {
    fn name(&self) -> &str { &self.name }
    fn symbol(&self) -> &str { &self.symbol }
    fn seller_fee_basis_points(&self) -> u16 { self.seller_fee_basis_points }
    fn creators(&self) -> &[CreatorData] { &self.creators }
}

impl<'info> NftUtils for CreateCollection<'info> {}

impl<'info> CreateCollection<'info> {
    fn validate_collection_data(&self, collection_data: &CollectionData) -> Result<()> {
        collection_data.validate()
    }
}

pub fn create_collection(
    ctx: Context<CreateCollection>,
    collection_data: CollectionData,
) -> Result<()> {
    ctx.accounts.validate_collection_data(&collection_data)?;
    
    let clock = Clock::get()?;

    let metadata = &ctx.accounts.metadata.to_account_info();
    let master_edition = &ctx.accounts.master_edition.to_account_info();
    let mint = &ctx.accounts.mint.to_account_info();
    let authority = &ctx.accounts.mint_authority.to_account_info();
    let payer = &ctx.accounts.user.to_account_info();
    let system_program = &ctx.accounts.system_program.to_account_info();
    let spl_token_program = &ctx.accounts.token_program.to_account_info();
    let spl_metadata_program = &ctx.accounts.token_metadata_program.to_account_info();

    let authority_bump = ctx.bumps.mint_authority;
    let seeds = &[&b"authority"[..], &[authority_bump]];
    let signer_seeds = &[&seeds[..]];

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_accounts = MintTo {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.destination.to_account_info(),
        authority: ctx.accounts.mint_authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    mint_to(cpi_ctx, 1).map_err(|_| NftError::InvalidName)?;
    msg!("Collection NFT minted!");

    let creators: Vec<Creator> = collection_data.creators
        .iter()
        .map(|creator_data| Creator {
            address: creator_data.address,
            verified: creator_data.verified,
            share: creator_data.share,
        })
        .collect();

    let metadata_account = CreateMetadataAccountV3Cpi::new(
        spl_metadata_program,
        CreateMetadataAccountV3CpiAccounts {
            metadata,
            mint,
            mint_authority: authority,
            payer,
            update_authority: (authority, true),
            system_program,
            rent: None,
        },
        CreateMetadataAccountV3InstructionArgs {
            data: DataV2 {
                name: collection_data.name.clone(),
                symbol: collection_data.symbol.clone(),
                uri: collection_data.uri.clone(),
                seller_fee_basis_points: collection_data.seller_fee_basis_points,
                creators: Some(creators),
                collection: None,
                uses: None,
            },
            is_mutable: true,
            collection_details: Some(mpl_token_metadata::types::CollectionDetails::V1 { size: 0 }),
        },
    );
    metadata_account
        .invoke_signed(signer_seeds)
        .map_err(|_| NftError::InvalidUri)?;
    msg!("Collection Metadata Account created!");

    let master_edition_account = CreateMasterEditionV3Cpi::new(
        spl_metadata_program,
        CreateMasterEditionV3CpiAccounts {
            edition: master_edition,
            update_authority: authority,
            mint_authority: authority,
            mint,
            payer,
            metadata,
            token_program: spl_token_program,
            system_program,
            rent: None,
        },
        CreateMasterEditionV3InstructionArgs {
            max_supply: Some(0),
        },
    );
    master_edition_account
        .invoke_signed(signer_seeds)
        .map_err(|_| NftError::InvalidUri)?;
    msg!("Collection Master Edition Account created");

    let name_bytes = ctx.accounts.string_to_bytes::<32>(&collection_data.name);
    let symbol_bytes = ctx.accounts.string_to_bytes::<10>(&collection_data.symbol);
    let uri_bytes = ctx.accounts.string_to_bytes::<200>(&collection_data.uri);

    let collection_info = &mut ctx.accounts.collection_info;
    collection_info.mint = ctx.accounts.mint.key();
    collection_info.name = name_bytes;   
    collection_info.symbol = symbol_bytes;   
    collection_info.uri = uri_bytes;        
    collection_info.creator = ctx.accounts.user.key();
    collection_info.created_at = clock.unix_timestamp;
    collection_info.number_of_nfts = 0;  
    collection_info.bump = ctx.bumps.collection_info;

    emit_cpi!(CollectionCreated {
        mint: ctx.accounts.mint.key(),
        name: collection_data.name,
        symbol: collection_data.symbol,
        uri: collection_data.uri,
        creator: ctx.accounts.user.key(),
        created_at: clock.unix_timestamp,
    });
    
    Ok(())
}