#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

#[derive(Clone)]
#[contracttype]
pub struct Bounty {
    pub id: u32,
    pub maintainer: Address,
    pub amount: i128,
    pub token: Address,
    pub is_claimed: bool,
    pub is_cancelled: bool,
    pub deadline: u64,          // Issue #7: Unix timestamp deadline, 0 = no deadline
    pub approved_claimant: Option<Address>, // Issue #6: Only this address can claim (after maintainer approval)
}

const BOUNTIES: Symbol = symbol_short!("BOUNTIES");
const NEXT_ID: Symbol = symbol_short!("NEXT_ID");

#[contract]
pub struct BountyContract;

#[contractimpl]
impl BountyContract {
    pub fn create_bounty(
        env: Env,
        maintainer: Address,
        amount: i128,
        token: Address,
        deadline: u64, // Issue #7: Unix timestamp. Auto-cancels if not claimed by this time.
    ) -> u32 {
        maintainer.require_auth();
        
        let id = env.storage().instance().get(&NEXT_ID).unwrap_or(1u32);
        env.storage().instance().set(&NEXT_ID, &(id + 1));
        
        let bounty = Bounty {
            id,
            maintainer: maintainer.clone(),
            amount,
            token: token.clone(),
            is_claimed: false,
            is_cancelled: false,
            deadline,
            approved_claimant: None,
        };
        
        let mut bounties: Vec<Bounty> = env.storage().instance().get(&BOUNTIES).unwrap_or(Vec::new(&env));
        bounties.push_back(bounty);
        env.storage().instance().set(&BOUNTIES, &bounties);
        
        // Transfer tokens from maintainer to contract
        let contract_address = env.current_contract_address();
        env.invoke_contract::<()>(
            &token,
            &symbol_short!("transfer"),
            soroban_sdk::vec![&env, maintainer.into_val(&env), contract_address.into_val(&env), amount.into_val(&env)]
        );
        
        id
    }
    
    // Issue #6: Maintainer approves a specific claimant before they can claim
    pub fn approve_claimant(
        env: Env,
        bounty_id: u32,
        claimant: Address,
    ) {
        let mut bounties: Vec<Bounty> = env.storage().instance().get(&BOUNTIES).unwrap_or(Vec::new(&env));

        for i in 0..bounties.len() {
            let mut bounty = bounties.get(i).unwrap();
            if bounty.id == bounty_id {
                bounty.maintainer.require_auth();

                if bounty.is_claimed {
                    panic!("Bounty already claimed");
                }
                if bounty.is_cancelled {
                    panic!("Bounty is cancelled");
                }
                if bounty.deadline > 0 && env.ledger().timestamp() > bounty.deadline {
                    panic!("Bounty deadline has passed");
                }

                bounty.approved_claimant = Some(claimant);
                bounties.set(i, bounty.clone());
                env.storage().instance().set(&BOUNTIES, &bounties);
                return;
            }
        }

        panic!("Bounty not found");
    }

    pub fn claim_bounty(
        env: Env,
        bounty_id: u32,
        claimant: Address,
    ) {
        claimant.require_auth();

        // Issue #7: Check deadline before processing
        let mut bounties: Vec<Bounty> = env.storage().instance().get(&BOUNTIES).unwrap_or(Vec::new(&env));

        for i in 0..bounties.len() {
            let mut bounty = bounties.get(i).unwrap();
            if bounty.id == bounty_id {
                // Issue #7: Auto-cancel if deadline passed
                if bounty.deadline > 0 && env.ledger().timestamp() > bounty.deadline {
                    if !bounty.is_claimed && !bounty.is_cancelled {
                        bounty.is_cancelled = true;
                        bounties.set(i, bounty.clone());
                        env.storage().instance().set(&BOUNTIES, &bounties);

                        // Return tokens to maintainer
                        env.invoke_contract::<()>(
                            &bounty.token,
                            &symbol_short!("transfer"),
                            soroban_sdk::vec![&env, env.current_contract_address().into_val(&env), bounty.maintainer.into_val(&env), bounty.amount.into_val(&env)],
                        );
                    }
                    panic!("Bounty deadline has passed and bounty is auto-cancelled");
                }

                if bounty.is_claimed {
                    panic!("Bounty already claimed");
                }
                if bounty.is_cancelled {
                    panic!("Bounty is cancelled");
                }

                // Issue #6: Only approved claimant can claim
                if let Some(ref approved) = bounty.approved_claimant {
                    if approved != &claimant {
                        panic!("Claimant not approved by maintainer");
                    }
                } else {
                    panic!("Claimant not approved by maintainer");
                }

                bounty.is_claimed = true;
                bounties.set(i, bounty.clone());
                env.storage().instance().set(&BOUNTIES, &bounties);

                // Transfer tokens to claimant
                env.invoke_contract::<()>(
                    &bounty.token,
                    &symbol_short!("transfer"),
                    soroban_sdk::vec![&env, env.current_contract_address().into_val(&env), claimant.into_val(&env), bounty.amount.into_val(&env)],
                );

                return;
            }
        }

        panic!("Bounty not found");
    }
    
    pub fn cancel_bounty(
        env: Env,
        bounty_id: u32,
    ) {
        let mut bounties: Vec<Bounty> = env.storage().instance().get(&BOUNTIES).unwrap_or(Vec::new(&env));

        for i in 0..bounties.len() {
            let mut bounty = bounties.get(i).unwrap();
            if bounty.id == bounty_id {
                bounty.maintainer.require_auth();

                if bounty.is_claimed {
                    panic!("Cannot cancel claimed bounty");
                }
                if bounty.is_cancelled {
                    panic!("Bounty already cancelled");
                }

                bounty.is_cancelled = true;
                bounties.set(i, bounty.clone());
                env.storage().instance().set(&BOUNTIES, &bounties);

                // Return tokens to maintainer
                env.invoke_contract::<()>(
                    &bounty.token,
                    &symbol_short!("transfer"),
                    soroban_sdk::vec![&env, env.current_contract_address().into_val(&env), bounty.maintainer.into_val(&env), bounty.amount.into_val(&env)],
                );

                return;
            }
        }

        panic!("Bounty not found");
    }
    
    pub fn get_bounty(env: Env, bounty_id: u32) -> Bounty {
        let bounties: Vec<Bounty> = env.storage().instance().get(&BOUNTIES).unwrap_or(Vec::new(&env));

        for i in 0..bounties.len() {
            let bounty = bounties.get(i).unwrap();
            if bounty.id == bounty_id {
                return bounty;
            }
        }

        panic!("Bounty not found");
    }

    pub fn get_all_bounties(env: Env) -> Vec<Bounty> {
        env.storage().instance().get(&BOUNTIES).unwrap_or(Vec::new(&env))
    }

    // Issue #7: Check and auto-cancel expired bounties (call this on chain to trigger)
    pub fn check_deadlines(env: Env) {
        let mut bounties: Vec<Bounty> = env.storage().instance().get(&BOUNTIES).unwrap_or(Vec::new(&env));
        let current_time = env.ledger().timestamp();

        for i in 0..bounties.len() {
            let mut bounty = bounties.get(i).unwrap();
            if bounty.deadline > 0 && current_time > bounty.deadline {
                if !bounty.is_claimed && !bounty.is_cancelled {
                    bounty.is_cancelled = true;
                    bounties.set(i, bounty.clone());

                    // Return tokens to maintainer
                    env.invoke_contract::<()>(
                        &bounty.token,
                        &symbol_short!("transfer"),
                        soroban_sdk::vec![&env, env.current_contract_address().into_val(&env), bounty.maintainer.into_val(&env), bounty.amount.into_val(&env)],
                    );
                }
            }
        }

        env.storage().instance().set(&BOUNTIES, &bounties);
    }
}
