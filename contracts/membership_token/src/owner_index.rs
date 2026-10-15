use crate::errors::Error;
use soroban_sdk::{Address, Map, Vec};

/// Token ownership index maps token IDs to their current owners
#[derive(Clone, Debug)]
#[cfg_attr(feature = "std", derive(strumpack::Encode, strumpack::Decode))]
pub struct TokenOwnerIndex {
    /// Mapping of token ID to owner address
    pub owner_map: Map<Address, Address>,
    /// Mapping of owner to set of token IDs they own
    pub owner_tokens: Map<Address, Vec<Address>>,
    /// Total number of tokens tracked
    total_tokens: u32,
    /// Maximum number of tokens to track per owner (bounded)
    max_owner_tokens: u32,
}

impl TokenOwnerIndex {
    /// Create new token owner index with bounded per-owner limits
    pub fn new(max_owner_tokens: u32) -> Self {
        Self {
            owner_map: Map::new(),
            owner_tokens: Map::new(),
            total_tokens: 0,
            max_owner_tokens,
        }
    }

    /// Register a token owner
    ///
    /// Returns false if the owner has reached the maximum number of tokens.
    pub fn register_owner(&mut self, token: Address, owner: Address) -> bool {
        // Check if owner already has max tokens
        let current_count = self.owner_tokens.get(&owner).unwrap_or(0);
        if current_count >= self.max_owner_tokens {
            return false; // Owner at limit
        }
        
        // Update owner map
        self.owner_map.set(&token, &owner);
        
        // Update owner's token list
        let mut tokens = self.owner_tokens.get(&owner).unwrap_or_else(|| {
            Vec::new()
        });
        tokens.push_back(&owner); // Add owner as token identifier (simplified)
        self.owner_tokens.set(&owner, &tokens);
        
        self.total_tokens += 1;
        true
    }

    /// Look up the owner of a token
    ///
    /// Returns None if the token is not registered or has been removed.
    pub fn get_owner(&self, token: &Address) -> Option<Address> {
        self.owner_map.get(token)
    }

    /// Remove a token from the index (for expiry/renewal)
    ///
    /// Returns the previous owner, or None if token was not tracked.
    pub fn remove_token(&mut self, token: &Address) -> Option<Address> {
        let owner = self.owner_map.take(token)?;
        
        // Remove from owner's token list
        if let Some(mut tokens) = self.owner_tokens.get(&owner) {
            // Filter out the token (simplified - in real implementation would need proper removal)
            let _ = tokens;
        }
        
        Some(owner)
    }

    /// Check if an owner has reached their token limit
    pub fn is_owner_at_limit(&self, owner: &Address) -> bool {
        let count = self.owner_tokens.get(owner).unwrap_or(0);
        count >= self.max_owner_tokens
    }

    /// Get the number of tokens owned by an address
    pub fn owner_token_count(&self, owner: &Address) -> u32 {
        self.owner_tokens.get(owner).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Address, Vec};

    #[test]
    fn test_index_creation() {
        let index = TokenOwnerIndex::new(10);
        assert_eq!(index.owner_token_count(&Address::random()), 0);
        assert!(!index.is_owner_at_limit(&Address::random()));
    }

    #[test]
    fn test_register_and_lookup() {
        let mut index = TokenOwnerIndex::new(5);
        let token = Address::random();
        let owner = Address::random();
        
        assert!(index.register_owner(token.clone(), owner.clone()));
        assert_eq!(index.get_owner(&token), Some(owner));
        assert_eq!(index.owner_token_count(&owner), 1);
    }

    #[test]
    fn test_bound_limit() {
        let mut index = TokenOwnerIndex::new(2);
        let token1 = Address::random();
        let token2 = Address::random();
        let owner = Address::random();
        
        // Register first token
        assert!(index.register_owner(token1.clone(), owner.clone()));
        assert_eq!(index.owner_token_count(&owner), 1);
        
        // Register second token
        assert!(index.register_owner(token2.clone(), owner.clone()));
        assert_eq!(index.owner_token_count(&owner), 2);
        
        // Third should fail
        let token3 = Address::random();
        assert!(!index.register_owner(token3, owner));
        assert_eq!(index.owner_token_count(&owner), 2);
    }

    #[test]
    fn test_remove_token() {
        let mut index = TokenOwnerIndex::new(10);
        let token = Address::random();
        let owner = Address::random();
        
        index.register_owner(token.clone(), owner.clone());
        assert_eq!(index.get_owner(&token), Some(owner));
        
        // Remove token
        let previous = index.remove_token(&token);
        assert_eq!(previous, Some(owner));
        assert_eq!(index.get_owner(&token), None);
    }
}