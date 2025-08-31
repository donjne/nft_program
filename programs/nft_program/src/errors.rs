use super::*;

#[error_code]
pub enum NftError {
    #[msg("Invalid name - must be between 1 and 32 characters")]
    InvalidName,
    #[msg("Invalid symbol - must be between 1 and 10 characters")]
    InvalidSymbol,
    #[msg("Invalid URI format")]
    InvalidUri,
    #[msg("Invalid seller fee basis points - must be between 0 and 10000")]
    InvalidSellerFeeBasisPoints,
    #[msg("Creator shares must sum to 100")]
    InvalidCreatorShares,
    #[msg("Maximum of 5 creators allowed")]
    TooManyCreators,
    #[msg("NFT is already verified in this collection")]
    AlreadyVerified,
    #[msg("Collection mint does not exist or does not match NFT's collection")]
    InvalidCollectionMint,
}