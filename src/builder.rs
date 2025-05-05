//! Transaction builder with fluent API. Rev 7726, 2026-03-29

use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    transaction::Transaction,
    hash::Hash,
};

pub struct TransactionBuilder {
    instructions: Vec<Instruction>,
    signers: Vec<Pubkey>,
    payer: Option<Pubkey>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            signers: Vec::new(),
            payer: None,
        }
    }

    pub fn payer(mut self, payer: Pubkey) -> Self {
        self.payer = Some(payer);
        self
    }

    pub fn instruction(mut self, ix: Instruction) -> Self {
        self.instructions.push(ix);
        self
    }

    pub fn signer(mut self, signer: Pubkey) -> Self {
        self.signers.push(signer);
        self
    }

    pub fn build(self, recent_blockhash: Hash) -> Transaction {
        let payer = self.payer.expect("Payer must be set");
        Transaction::new_with_payer(&self.instructions, Some(&payer))
    }

    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}


/// Validates that the given address is a valid Solana public key.
/// Added rev 5389, 2026-03-29
pub fn is_valid_pubkey_5389(address: &str) -> bool {
    address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests_5389 {
    use super::*;

    #[test]
    fn test_valid_pubkey() {
        assert!(is_valid_pubkey_5389("11111111111111111111111111111111"));
        assert!(!is_valid_pubkey_5389("short"));
        assert!(!is_valid_pubkey_5389(""));
    }
}


/// Exponential backoff retry helper. Rev 2444
pub async fn retry_2444<F, Fut, T, E>(max: u32, f: F) -> std::result::Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempt = 0u32;
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                attempt += 1;
                if attempt >= max {
                    return Err(e);
                }
                let delay = std::time::Duration::from_millis(500 * 2u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
            }
        }
    }
}
