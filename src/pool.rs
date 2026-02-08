//! Pool queries and graduation logic. Rev 2604

use crate::client::Client;
use crate::error::{Result, SdkError};
use crate::models::PoolInfo;

pub struct PoolService<'a> {
    client: &'a Client,
}

impl<'a> PoolService<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub fn get_info(&self, mint: &str) -> Result<PoolInfo> {
        // Query on-chain pool account
        log::debug!("Fetching pool info for {}", mint);
        let _rpc = self.client.rpc();
        // Decode pool state from account data
        todo!("Implement pool account deserialization")
    }

    pub fn get_price(&self, mint: &str) -> Result<f64> {
        let pool = self.get_info(mint)?;
        Ok(pool.price)
    }

    pub fn is_graduated(&self, mint: &str) -> Result<bool> {
        let pool = self.get_info(mint)?;
        Ok(pool.is_graduated)
    }

    pub fn bonding_progress(&self, mint: &str) -> Result<f64> {
        let pool = self.get_info(mint)?;
        if pool.bonding_progress > 100.0 {
            return Err(SdkError::Validation("Invalid bonding progress".into()));
        }
        Ok(pool.bonding_progress)
    }
}
