use super::*;
use crate::instructions::shared::validation::ValidatableData;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NftData {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Vec<CreatorData>,
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(nft_data: NftData)]
pub struct MintNFT<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        payer = owner,
        mint::decimals = 0,
        mint::authority = mint_authority,
        mint::freeze_authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = owner,
        associated_token::mint = mint,
        associated_token::authority = owner
    )]
    pub destination: Account<'info, TokenAccount>,
    
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    /// CHECK: This account is used for signing purposes only
    pub mint_authority: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + NftInfo::INIT_SPACE,
        seeds = [b"nft", mint.key().as_ref()],
        bump,
    )]
    pub nft_info: Account<'info, NftInfo>,
    
    #[account(mut)]
    /// CHECK: This account will be initialized by the metaplex program
    pub metadata: UncheckedAccount<'info>,
    
    #[account(mut)]
    /// CHECK: This account will be initialized by the metaplex program
    pub master_edition: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub collection_mint: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
}

impl validation::ValidatableData for NftData {
    fn name(&self) -> &str { &self.name }
    fn symbol(&self) -> &str { &self.symbol }
    fn seller_fee_basis_points(&self) -> u16 { self.seller_fee_basis_points }
    fn creators(&self) -> &[CreatorData] { &self.creators }
}

impl<'info> NftUtils for MintNFT<'info> {}

impl<'info> MintNFT<'info> {
    fn validate_nft_data(&self, nft_data: &NftData) -> Result<()> {
        nft_data.validate()
    }
}

pub fn mint_nft(
    ctx: Context<MintNFT>,
    nft_data: NftData,
) -> Result<()> {
    ctx.accounts.validate_nft_data(&nft_data)?;
    
    let clock = Clock::get()?;

    let metadata = &ctx.accounts.metadata.to_account_info();
    let master_edition = &ctx.accounts.master_edition.to_account_info();
    let mint = &ctx.accounts.mint.to_account_info();
    let authority = &ctx.accounts.mint_authority.to_account_info();
    let payer = &ctx.accounts.owner.to_account_info();
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
    msg!("NFT minted!");
    
    // Convert CreatorData to Creator
    let creators: Vec<Creator> = nft_data.creators
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
                name: nft_data.name.clone(),
                symbol: nft_data.symbol.clone(),
                uri: nft_data.uri.clone(),
                seller_fee_basis_points: nft_data.seller_fee_basis_points,
                creators: Some(creators),
                collection: Some(Collection {
                    verified: false,
                    key: ctx.accounts.collection_mint.key(),
                }),
                uses: None,
            },
            is_mutable: true,
            collection_details: None,
        },
    );
    metadata_account
        .invoke_signed(signer_seeds)
        .map_err(|_| NftError::InvalidUri)?;
    msg!("NFT Metadata Account created!");

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
    msg!("NFT Master Edition Account created");

    let name_bytes = ctx.accounts.string_to_bytes::<32>(&nft_data.name);
    let symbol_bytes = ctx.accounts.string_to_bytes::<10>(&nft_data.symbol);
    let uri_bytes = ctx.accounts.string_to_bytes::<200>(&nft_data.uri);
    
    let nft_info = &mut ctx.accounts.nft_info;
    nft_info.mint = ctx.accounts.mint.key();
    nft_info.collection_mint = ctx.accounts.collection_mint.key();
    nft_info.name = name_bytes;         
    nft_info.symbol = symbol_bytes;      
    nft_info.uri = uri_bytes;         
    nft_info.owner = ctx.accounts.owner.key();
    nft_info.minted_at = clock.unix_timestamp;
    nft_info.verified = false;  
    nft_info.bump = ctx.bumps.nft_info;

    emit_cpi!(NftMinted {
        mint: ctx.accounts.mint.key(),
        collection_mint: ctx.accounts.collection_mint.key(),
        name: nft_data.name,
        symbol: nft_data.symbol,
        uri: nft_data.uri,
        owner: ctx.accounts.owner.key(),
        minted_at: clock.unix_timestamp,
    });
    
    Ok(())
}