use super::*;

#[derive(InitSpace)]
#[account]
pub struct CollectionInfo {
    pub mint: Pubkey,
    pub name: [u8; 32],
    pub symbol: [u8; 10],   
    pub uri: [u8; 200], 
    pub creator: Pubkey,
    pub number_of_nfts: u64,
    pub created_at: i64,
    pub bump: u8,
}