use super::*;

#[derive(InitSpace)]
#[account]
pub struct NftInfo {
    pub mint: Pubkey,
    pub collection_mint: Pubkey,
    pub name: [u8; 32],   
    pub symbol: [u8; 10], 
    pub uri: [u8; 200],     
    pub owner: Pubkey,
    pub verified: bool,
    pub minted_at: i64,
    pub bump: u8,
}
