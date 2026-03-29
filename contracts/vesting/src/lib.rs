#![no_std]


#[contract]
pub struct VestingContract;

#[contractimpl]
impl VestingContract {


        claimable
    }

    /// Revoke the vesting schedule.
main
    pub fn revoke(env: Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();


    }
}
