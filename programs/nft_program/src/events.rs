use super::*;

#[event]
pub struct CollectionCreated {
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub creator: Pubkey,
    pub created_at: i64,
}

#[event]
pub struct NftMinted {
    pub mint: Pubkey,
    pub collection_mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub owner: Pubkey,
    pub minted_at: i64,
}

#[event]
pub struct CollectionVerified {
    pub nft_mint: Pubkey,
    pub collection_mint: Pubkey,
    pub authority: Pubkey,
    pub verified_at: i64,
}
