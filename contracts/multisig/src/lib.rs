#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, Env,
    Symbol, Vec, Val, xdr::ToXdr,
};

#[cfg(test)]
mod test;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    InvalidThreshold = 3,
    InsufficientSigners = 4,
    Unauthorized = 5,
    ProposalNotFound = 6,
    AlreadyApproved = 7,
    ThresholdNotMet = 8,
    AlreadyExecuted = 9,
    AlreadyCancelled = 10,
}

#[contracttype]
pub enum DataKey {
    Signers,
    Threshold,
    Proposal(Bytes),           // Proposal Hash -> Info
    Approval(Bytes, Address), // (Hash, Signer) -> bool
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ProposalInfo {
    pub approval_count: u32,
    pub executed: bool,
    pub cancelled: bool,
}

#[contract]
pub struct MultisigWallet;

#[contractimpl]
impl MultisigWallet {
    /// Initialize the multisig wallet with a list of signers and a threshold.
    pub fn init(env: Env, signers: Vec<Address>, threshold: u32) {
        if env.storage().instance().has(&DataKey::Threshold) {
            env.panic_with_error(Error::AlreadyInitialized);
        }
        if threshold == 0 || threshold > signers.len() {
            env.panic_with_error(Error::InvalidThreshold);
        }

        env.storage().instance().set(&DataKey::Signers, &signers);
        env.storage().instance().set(&DataKey::Threshold, &threshold);
    }

    /// Create a new proposal.
    pub fn propose(env: Env, target: Address, function: Symbol, args: Vec<Val>, salt: Bytes) -> Bytes {
        let hash = Self::calculate_hash(&env, &target, &function, &args, &salt);
        
        if env.storage().persistent().has(&DataKey::Proposal(hash.clone())) {
            // Proposal already exists, no need to overwrite but could allow re-proposing if cancelled?
            return hash;
        }

        let info = ProposalInfo {
            approval_count: 0,
            executed: false,
            cancelled: false,
        };

        env.storage().persistent().set(&DataKey::Proposal(hash.clone()), &info);
        
        env.events().publish(
            (symbol_short!("proposed"), hash.clone()),
            (target, function)
        );

        hash
    }

    /// Approve a proposal.
    pub fn approve(env: Env, signer: Address, hash: Bytes) {
        signer.require_auth();
        
        // Verify signer
        let signers: Vec<Address> = env.storage().instance().get(&DataKey::Signers).unwrap();
        if !signers.contains(&signer) {
            env.panic_with_error(Error::Unauthorized);
        }

        let mut info: ProposalInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(hash.clone()))
            .unwrap_or_else(|| env.panic_with_error(Error::ProposalNotFound));

        if info.executed {
            env.panic_with_error(Error::AlreadyExecuted);
        }
        if info.cancelled {
            env.panic_with_error(Error::AlreadyCancelled);
        }

        let approval_key = DataKey::Approval(hash.clone(), signer.clone());
        if env.storage().persistent().has(&approval_key) {
            env.panic_with_error(Error::AlreadyApproved);
        }

        info.approval_count += 1;
        env.storage().persistent().set(&DataKey::Proposal(hash.clone()), &info);
        env.storage().persistent().set(&approval_key, &true);

        env.events().publish((symbol_short!("approved"), hash), signer.clone());
    }

    /// Execute a proposal if the threshold is met.
    pub fn execute(env: Env, target: Address, function: Symbol, args: Vec<Val>, salt: Bytes) -> Val {
        let hash = Self::calculate_hash(&env, &target, &function, &args, &salt);
        
        let mut info: ProposalInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(hash.clone()))
            .unwrap_or_else(|| env.panic_with_error(Error::ProposalNotFound));

        let threshold: u32 = env.storage().instance().get(&DataKey::Threshold).unwrap();

        if info.approval_count < threshold {
            env.panic_with_error(Error::ThresholdNotMet);
        }
        if info.executed {
            env.panic_with_error(Error::AlreadyExecuted);
        }
        if info.cancelled {
            env.panic_with_error(Error::AlreadyCancelled);
        }

        info.executed = true;
        env.storage().persistent().set(&DataKey::Proposal(hash.clone()), &info);

        let result = env.invoke_contract::<Val>(&target, &function, args);
        env.events().publish((symbol_short!("executed"), hash), target);
        result
    }

    /// Cancel a proposal. Can be called by the multisig itself (governance) or a proposer?
    /// For simplicity, let's allow any signer to cancel if they can get a majority? 
    /// Or just the multisig itself can cancel (meaning a proposal to cancel).
    pub fn cancel(env: Env, hash: Bytes) {
        env.current_contract_address().require_auth();

        let mut info: ProposalInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(hash.clone()))
            .unwrap_or_else(|| env.panic_with_error(Error::ProposalNotFound));

        if info.executed {
            env.panic_with_error(Error::AlreadyExecuted);
        }

        info.cancelled = true;
        env.storage().persistent().set(&DataKey::Proposal(hash.clone()), &info);

        env.events().publish((symbol_short!("cancelled"), hash), ());
    }

    // --- Admin functions (must be called via the multisig itself) ---

    pub fn add_signer(env: Env, signer: Address) {
        env.current_contract_address().require_auth();
        let mut signers: Vec<Address> = env.storage().instance().get(&DataKey::Signers).unwrap();
        if !signers.contains(&signer) {
            signers.push_back(signer);
            env.storage().instance().set(&DataKey::Signers, &signers);
        }
    }

    pub fn remove_signer(env: Env, signer: Address) {
        env.current_contract_address().require_auth();
        let mut signers: Vec<Address> = env.storage().instance().get(&DataKey::Signers).unwrap();
        let threshold: u32 = env.storage().instance().get(&DataKey::Threshold).unwrap();

        if let Some(idx) = signers.first_index_of(&signer) {
            if signers.len() <= threshold {
                env.panic_with_error(Error::InvalidThreshold);
            }
            signers.remove(idx);
            env.storage().instance().set(&DataKey::Signers, &signers);
        }
    }

    pub fn set_threshold(env: Env, threshold: u32) {
        env.current_contract_address().require_auth();
        let signers: Vec<Address> = env.storage().instance().get(&DataKey::Signers).unwrap();
        if threshold == 0 || threshold > signers.len() {
            env.panic_with_error(Error::InvalidThreshold);
        }
        env.storage().instance().set(&DataKey::Threshold, &threshold);
    }

    // --- Internal Helpers ---
    fn calculate_hash(
        env: &Env,
        target: &Address,
        function: &Symbol,
        args: &Vec<Val>,
        salt: &Bytes,
    ) -> Bytes {
        let mut data = Bytes::new(env);
        data.append(&target.clone().to_xdr(env));
        data.append(&function.clone().to_xdr(env));
        data.append(&args.clone().to_xdr(env));
        data.append(&salt.clone().to_xdr(env));
        
        env.crypto().sha256(&data).into()
    }
}
