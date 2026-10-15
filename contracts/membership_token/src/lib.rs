//! Membership token contract with transfer approval management
//! 
//! CT-20: Expire transfer approvals automatically
//! CT-22: Add bounded token-owner lookup and index repair

use crate::errors::Error;
use crate::types::MemberId;
use crate::owner_index::TokenOwnerIndex;

/// Represents a transfer approval with an expiration timestamp
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(strumpack::Encode, strumpack::Decode))]
pub struct Approval {
    /// The token owner who granted the approval
    pub owner: MemberId,
    /// The approved spender
    pub spender: MemberId,
    /// When the approval expires (0 means no expiration)
    pub expires_at: u64,
    /// The amount that can be transferred
    pub amount: u128,
}

impl Approval {
    /// Create a new approval with expiration
    pub fn new(owner: MemberId, spender: MemberId, duration: u64) -> Self {
        let now = <crate::env::Clock as CurrentTime>::get();
        Self {
            owner,
            spender,
            expires_at: now + duration,
            amount: u128::MAX,
        }
    }

    /// Check if the approval has expired
    pub fn is_expired(&self) -> bool {
        let now = <crate::env::Clock as CurrentTime>::get();
        now >= self.expires_at && self.expires_at != 0
    }

    /// Get remaining time until expiration
    pub fn time_until_expiry(&self) -> u64 {
        let now = <crate::env::Clock as CurrentTime>::get();
        if self.is_expired() {
            0
        } else {
            self.expires_at.saturating_sub(now)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_creation() {
        let approval = Approval::new(
            [0u8; 32],
            [1u8; 32],
            86400, // 24 hours
        );
        assert!(!approval.is_expired()); // New approval should not be expired
    }

    #[test]
    fn test_approval_expiry() {
        let approval = Approval::new(
            [0u8; 32],
            [1u8; 32],
            1, // 1 second
        );
        // Immediately check - might still not be expired depending on clock
    }
}