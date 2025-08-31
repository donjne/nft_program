use super::*;

pub trait NftUtils {
    fn string_to_bytes<const N: usize>(&self, s: &str) -> [u8; N] {
        let mut bytes = [0u8; N];
        let s_bytes = s.as_bytes();
        let len = s_bytes.len().min(N);
        bytes[..len].copy_from_slice(&s_bytes[..len]);
        bytes
    }

    fn bytes_to_string<const N: usize>(&self, bytes: &[u8; N]) -> String {
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(N);
        String::from_utf8_lossy(&bytes[..end]).to_string()
    }
}

pub mod validation {
    use super::*;

    pub fn validate_name(name: &str) -> Result<()> {
        if name.is_empty() || name.len() > 32 {
            return Err(error!(NftError::InvalidName));
        }
        Ok(())
    }

    pub fn validate_symbol(symbol: &str) -> Result<()> {
        if symbol.is_empty() || symbol.len() > 10 {
            return Err(error!(NftError::InvalidSymbol));
        }
        Ok(())
    }

    pub fn validate_seller_fee_basis_points(fee: u16) -> Result<()> {
        if fee > 10000 {
            return Err(error!(NftError::InvalidSellerFeeBasisPoints));
        }
        Ok(())
    }

    pub fn validate_creators(creators: &[CreatorData]) -> Result<()> {
        if creators.len() > 5 {
            return Err(error!(NftError::TooManyCreators));
        }
        
        let total_share: u16 = creators.iter().map(|c| c.share as u16).sum();
        if total_share != 100 {
            return Err(error!(NftError::InvalidCreatorShares));
        }
        
        Ok(())
    }

    pub trait ValidatableData {
        fn name(&self) -> &str;
        fn symbol(&self) -> &str;
        fn seller_fee_basis_points(&self) -> u16;
        fn creators(&self) -> &[CreatorData];

        fn validate(&self) -> Result<()> {
            validate_name(self.name())?;
            validate_symbol(self.symbol())?;
            validate_seller_fee_basis_points(self.seller_fee_basis_points())?;
            validate_creators(self.creators())?;
            Ok(())
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CreatorData {
    pub address: Pubkey,
    pub verified: bool,
    pub share: u8,
}