use marinade_finance_offchain_sdk::anchor_lang::prelude::*;
use marinade_finance_offchain_sdk::{
    instruction_helpers::{initialize::InitializeBuilder, InstructionHelpers},
    marinade_finance::{liq_pool::LiqPool, Fee, State, MAX_REWARD_FEE},
    transaction_builder::TransactionBuilder,
};
use std::sync::Arc;

use lazy_static::lazy_static;
use marinade_finance_offchain_sdk::spl_associated_token_account::get_associated_token_address;
use rand::{distributions::Uniform, prelude::*};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

lazy_static! {
    static ref CREATOR_AUTHORITY: Arc<Keypair> = Arc::new(
        Keypair::from_bytes(&[
            32, 187, 49, 197, 126, 48, 103, 246, 20, 101, 34, 108, 27, 43, 10, 147, 242, 239, 28,
            37, 66, 146, 103, 94, 29, 106, 142, 73, 10, 103, 249, 56, 130, 33, 92, 198, 248, 0, 48,
            210, 221, 172, 150, 104, 107, 227, 44, 217, 3, 61, 74, 58, 179, 76, 35, 104, 39, 67,
            130, 92, 93, 25, 180, 107
        ])
        .unwrap()
    );
}

pub trait InitializeInput {
    fn state(&self) -> Arc<Keypair>;
    fn msol_mint(&self) -> Arc<Keypair>;
    fn admin_authority(&self) -> Arc<Keypair>;
    fn operational_sol_account(&self) -> Pubkey;
    fn validator_manager_authority(&self) -> Arc<Keypair>;
    fn treasury_msol(&self) -> Pubkey;
    fn treasury_msol_authority(&self) -> Pubkey;
    fn build_treasury_msol_account(&self, builder: &mut InitializeBuilder);
    fn lp_mint(&self) -> Arc<Keypair>;
    fn max_stake_count(&self) -> u32;
    fn max_validator_count(&self) -> u32;
    fn reward_fee(&self) -> Fee;
    fn stake_list(&self) -> Pubkey;
    fn build_stake_list(&self, builder: &mut InitializeBuilder, rent: &Rent);
    fn validator_list(&self) -> Pubkey;
    fn build_validator_list(&self, builder: &mut InitializeBuilder, rent: &Rent);
    fn liq_pool_msol_leg(&self) -> Pubkey;
    fn build_liq_pool_msol_leg(&self, builder: &mut InitializeBuilder, rent: &Rent);

    fn build(&self, builder: TransactionBuilder, rent: &Rent) -> TransactionBuilder {
        let mut builder = builder
            .initialize(self.state(), CREATOR_AUTHORITY.clone())
            .unwrap();
        builder.create_msol_mint(self.msol_mint(), &rent);
        builder.set_admin_authority(self.admin_authority().pubkey());
        builder.set_operational_sol_account(self.operational_sol_account());
        builder.use_validator_manager_authority(self.validator_manager_authority().pubkey());
        self.build_treasury_msol_account(&mut builder);
        builder.set_reward_fee(self.reward_fee());
        builder.init_reserve(0, &rent).unwrap();
        builder.create_lp_mint(self.lp_mint(), &rent);
        builder.init_liq_pool_sol_leg(0, &rent).unwrap();

        self.build_stake_list(&mut builder, rent);
        self.build_validator_list(&mut builder, rent);
        self.build_liq_pool_msol_leg(&mut builder, rent);
        builder.build(rent)
    }

    // fn expected_reflection(&self, rent: &Rent, _clock: &Clock) -> Marinade {
    //     let mut builder = marinade_reflection::builder::Builder::default();
    //     let rent_exempt_for_token_acc = rent.minimum_balance(spl_token::state::Account::LEN);
    //     builder.set_msol_mint(self.msol_mint().pubkey());
    //     builder.set_admin_authority(self.admin_authority().pubkey());
    //     builder.set_operational_sol_account(self.operational_sol_account());
    //     builder.set_treasury_msol_account(self.treasury_msol());
    //     builder.set_min_stake(LAMPORTS_PER_SOL);
    //     builder.set_reward_fee(self.reward_fee());
    //     builder.set_validator_manager_authority(self.validator_manager_authority().pubkey());
    //     builder.set_free_validator_slots(self.max_validator_count()); // no used validators and free slots == max validators
    //     builder.set_total_cooling_down(0);
    //     builder.set_cooling_down_stakes(0);
    //     builder.set_free_stake_slots(self.max_stake_count()); // no used stakes and free slots == max stakes
    //     builder.set_lp_mint(self.lp_mint().pubkey());
    //     builder.set_lp_supply(0);
    //     builder.set_actual_liq_pool_sol_amount(rent_exempt_for_token_acc);
    //     builder.set_actual_liq_pool_msol_amount(0);
    //     builder.set_lp_liquidity_target(10000 * LAMPORTS_PER_SOL);
    //     builder.set_lp_max_fee(Fee::from_basis_points(300));
    //     builder.set_lp_min_fee(Fee::from_basis_points(30));
    //     builder.set_lp_treasury_cut(Fee::from_basis_points(2500));
    //     builder.set_available_reserve_balance(0);
    //     builder.set_msol_supply(0);
    //     builder.set_slots_for_stake_delta(24000);
    //     builder.set_last_stake_delta_epoch(Epoch::MAX);
    //     builder.set_min_deposit(1);
    //     builder.set_min_withdraw(1);
    //     builder.build(rent)
    // }
}

// Guide for with seed creation process
pub struct InitializeInputWithSeeds {
    pub state: Arc<Keypair>,
    pub msol_mint: Arc<Keypair>,
    pub admin_authority: Arc<Keypair>,
    pub operational_sol_account: Pubkey,
    pub validator_manager_authority: Arc<Keypair>,
    pub treasury_msol_authority: Pubkey,
    pub lp_mint: Arc<Keypair>,

    pub max_stake_count: u32,
    pub max_validator_count: u32,
    pub reward_fee: Fee,
}

impl InitializeInput for InitializeInputWithSeeds {
    fn state(&self) -> Arc<Keypair> {
        self.state.clone()
    }

    fn msol_mint(&self) -> Arc<Keypair> {
        self.msol_mint.clone()
    }

    fn admin_authority(&self) -> Arc<Keypair> {
        self.admin_authority.clone()
    }

    fn operational_sol_account(&self) -> Pubkey {
        self.operational_sol_account
    }

    fn validator_manager_authority(&self) -> Arc<Keypair> {
        self.validator_manager_authority.clone()
    }

    fn treasury_msol(&self) -> Pubkey {
        get_associated_token_address(&self.treasury_msol_authority(), &self.msol_mint().pubkey())
    }

    fn treasury_msol_authority(&self) -> Pubkey {
        self.treasury_msol_authority
    }

    fn build_treasury_msol_account(&self, builder: &mut InitializeBuilder) {
        builder.create_treasury_msol_account(self.treasury_msol_authority());
    }

    fn lp_mint(&self) -> Arc<Keypair> {
        self.lp_mint.clone()
    }

    fn max_stake_count(&self) -> u32 {
        self.max_stake_count
    }

    fn max_validator_count(&self) -> u32 {
        self.max_validator_count
    }

    fn reward_fee(&self) -> Fee {
        self.reward_fee
    }

    fn stake_list(&self) -> Pubkey {
        State::default_stake_list_address(&self.state.pubkey())
    }

    fn build_stake_list(&self, builder: &mut InitializeBuilder, rent: &Rent) {
        builder.create_stake_list_with_seed(self.max_stake_count, &rent);
    }

    fn validator_list(&self) -> Pubkey {
        State::default_validator_list_address(&self.state.pubkey())
    }

    fn build_validator_list(&self, builder: &mut InitializeBuilder, rent: &Rent) {
        builder.create_validator_list_with_seed(self.max_validator_count, rent);
    }

    fn liq_pool_msol_leg(&self) -> Pubkey {
        LiqPool::default_msol_leg_address(&self.state.pubkey())
    }

    fn build_liq_pool_msol_leg(&self, builder: &mut InitializeBuilder, rent: &Rent) {
        builder.create_liq_pool_msol_leg_with_seed(rent);
    }
}

impl InitializeInputWithSeeds {
    pub fn random<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        Self {
            state: Arc::new(Keypair::generate(rng)),
            msol_mint: Arc::new(Keypair::generate(rng)),
            admin_authority: Arc::new(Keypair::generate(rng)),
            operational_sol_account: Pubkey::new_unique(),
            validator_manager_authority: Arc::new(Keypair::generate(rng)),
            treasury_msol_authority: Pubkey::new_unique(),
            lp_mint: Arc::new(Keypair::generate(rng)),
            max_stake_count: Uniform::from(10..=100).sample(rng),
            max_validator_count: Uniform::from(1..=20).sample(rng),
            reward_fee: Fee::from_basis_points(Uniform::from(0..=MAX_REWARD_FEE).sample(rng)),
        }
    }
}

// Guide for without seed creation process
pub struct InitializeInputWithoutSeeds {
    pub with_seeds: InitializeInputWithSeeds,
    pub stake_list: Arc<Keypair>,
    pub validator_list: Arc<Keypair>,
    pub liq_pool_msol_leg: Arc<Keypair>,
}

impl InitializeInput for InitializeInputWithoutSeeds {
    fn state(&self) -> Arc<Keypair> {
        self.with_seeds.state()
    }

    fn msol_mint(&self) -> Arc<Keypair> {
        self.with_seeds.msol_mint()
    }

    fn admin_authority(&self) -> Arc<Keypair> {
        self.with_seeds.admin_authority()
    }

    fn operational_sol_account(&self) -> Pubkey {
        self.with_seeds.operational_sol_account()
    }

    fn validator_manager_authority(&self) -> Arc<Keypair> {
        self.with_seeds.validator_manager_authority()
    }

    fn treasury_msol(&self) -> Pubkey {
        get_associated_token_address(&self.treasury_msol_authority(), &self.msol_mint().pubkey())
    }

    fn treasury_msol_authority(&self) -> Pubkey {
        self.with_seeds.treasury_msol_authority()
    }

    fn build_treasury_msol_account(&self, builder: &mut InitializeBuilder) {
        builder.create_treasury_msol_account(self.treasury_msol_authority());
    }

    fn lp_mint(&self) -> Arc<Keypair> {
        self.with_seeds.lp_mint()
    }

    fn max_stake_count(&self) -> u32 {
        self.with_seeds.max_stake_count()
    }

    fn max_validator_count(&self) -> u32 {
        self.with_seeds.max_validator_count()
    }

    fn reward_fee(&self) -> Fee {
        self.with_seeds.reward_fee()
    }

    fn stake_list(&self) -> Pubkey {
        self.stake_list.pubkey()
    }

    fn build_stake_list(&self, builder: &mut InitializeBuilder, rent: &Rent) {
        builder.create_stake_list(self.stake_list.clone(), self.max_stake_count(), rent);
    }

    fn validator_list(&self) -> Pubkey {
        self.validator_list.pubkey()
    }

    fn build_validator_list(&self, builder: &mut InitializeBuilder, rent: &Rent) {
        builder.create_validator_list(
            self.validator_list.clone(),
            self.max_validator_count(),
            rent,
        );
    }

    fn liq_pool_msol_leg(&self) -> Pubkey {
        self.liq_pool_msol_leg.pubkey()
    }

    fn build_liq_pool_msol_leg(&self, builder: &mut InitializeBuilder, rent: &Rent) {
        builder.create_liq_pool_msol_leg(self.liq_pool_msol_leg.clone(), rent);
    }
}

impl InitializeInputWithoutSeeds {
    pub fn random<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        Self {
            with_seeds: InitializeInputWithSeeds::random(rng),
            stake_list: Arc::new(Keypair::generate(rng)),
            validator_list: Arc::new(Keypair::generate(rng)),
            liq_pool_msol_leg: Arc::new(Keypair::generate(rng)),
        }
    }
}
