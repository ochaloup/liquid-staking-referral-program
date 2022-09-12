use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;
use anchor_spl::token::TokenAccount;

use crate::constant::*;
use crate::error::ReferralError::ReferralOperationFeeOverMax;
use crate::error::*;
use crate::states::{GlobalState, ReferralState};

//-----------------------------------------------------
#[derive(Accounts)]
pub struct Initialize<'info> {
    // admin account
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    // address = Pubkey::from_str(GLOBAL_STATE_ADDRESS).unwrap(),
    #[account(zero)]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // mSOL treasury account for this referral program (must be fed externally)
    // owner must be self.global_state.get_treasury_auth()
    #[account()]
    pub treasury_msol_account: CpiAccount<'info, TokenAccount>,
}
impl<'info> Initialize<'info> {
    pub fn process(&mut self, treasury_msol_auth_bump: u8) -> ProgramResult {
        msg!("Initialize admin account: {}, global state account: {}", self.admin_account.key(), self.global_state.key());
        self.global_state.admin_account = self.admin_account.key();
        self.global_state.treasury_msol_auth_bump = treasury_msol_auth_bump;
        self.global_state.treasury_msol_account = self.treasury_msol_account.key();

        // verify the treasury account auth is this program get_treasury_auth() PDA (based on treasury_msol_auth_bump)
        if self.treasury_msol_account.owner != self.global_state.get_treasury_auth() {
            msg!("Treasure msol account owner: {}, treaasure auth: {}", self.treasury_msol_account.owner, self.global_state.get_treasury_auth());
            return Err(ReferralError::TreasuryTokenAuthorityDoesNotMatch.into());
        }

        if self.treasury_msol_account.delegate.is_some() {
            return Err(ReferralError::TreasuryTokenAccountMustNotBeDelegated.into());
        }

        if self.treasury_msol_account.close_authority.is_some() {
            return Err(ReferralError::TreasuryTokenAccountMustNotBeCloseable.into());
        }

        Ok(())
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct InitReferralAccount<'info> {
    // global state
    // address = Pubkey::from_str(GLOBAL_STATE_ADDRESS).unwrap(),
    #[account(
    has_one = admin_account,
        has_one = treasury_msol_account,
    )]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // admin account, signer
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    // mSOL treasury account for this referral program (added here to check partner token mint)
    #[account()]
    pub treasury_msol_account: CpiAccount<'info, TokenAccount>,

    #[account(zero)] // must be created but empty, ready to be initialized
    pub referral_state: ProgramAccount<'info, ReferralState>,

    // partner main account
    #[account()]
    pub partner_account: AccountInfo<'info>,

    // partner beneficiary mSOL ATA
    #[account()]
    pub token_partner_account: CpiAccount<'info, TokenAccount>,
}

impl<'info> InitReferralAccount<'info> {
    pub fn process(&mut self, partner_name: String) -> ProgramResult {
        msg!("process_init_referral_account");
        if partner_name.len() > 20 {
            msg!("max partner_name.len() is 20");
            return Err(ReferralError::PartnerNameTooLong.into());
        }

        // check if beneficiary account address matches to partner_address and msol_mint
        if self.token_partner_account.owner != *self.partner_account.key {
            return Err(ReferralError::InvalidBeneficiaryAccountOwner.into());
        }

        // verify the partner token account mint equals to treasury_msol_account
        if self.token_partner_account.mint != self.treasury_msol_account.mint {
            return Err(ReferralError::InvalidBeneficiaryAccountMint.into());
        }

        self.referral_state.partner_name = partner_name.clone();

        self.referral_state.partner_account = self.partner_account.key();
        self.referral_state.token_partner_account = self.token_partner_account.key();

        self.referral_state.transfer_duration = DEFAULT_TRANSFER_DURATION;
        self.referral_state.last_transfer_time = clock::Clock::get().unwrap().unix_timestamp;

        self.referral_state.deposit_sol_amount = 0;
        self.referral_state.deposit_sol_operations = 0;

        self.referral_state.deposit_stake_account_amount = 0;
        self.referral_state.deposit_stake_account_operations = 0;

        self.referral_state.liq_unstake_msol_amount = 0;
        self.referral_state.liq_unstake_operations = 0;
        self.referral_state.liq_unstake_msol_fees = 0;

        self.referral_state.delayed_unstake_amount = 0;
        self.referral_state.delayed_unstake_operations = 0;

        self.referral_state.max_net_stake = DEFAULT_MAX_NET_STAKE;
        self.referral_state.base_fee = DEFAULT_BASE_FEE_POINTS;
        self.referral_state.max_fee = DEFAULT_MAX_FEE_POINTS;

        self.referral_state.pause = false;

        self.referral_state.max_operation_fee = DEFAULT_MAX_OPERATION_FEE_POINTS;
        self.referral_state.operation_deposit_sol_fee = 0;
        self.referral_state.operation_deposit_stake_account_fee = 0;
        self.referral_state.operation_liquid_unstake_fee = 0;

        Ok(())
    }
}

//--------------------------------------
#[derive(Accounts)]
pub struct ChangeAuthority<'info> {
    // global state
    // address = Pubkey::from_str(GLOBAL_STATE_ADDRESS).unwrap(),
    #[account(
    mut,
        has_one = admin_account,
    )]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // current admin account (must match the one in GlobalState)
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    // new admin account
    pub new_admin_account: AccountInfo<'info>,
}
impl<'info> ChangeAuthority<'info> {
    pub fn process(&mut self) -> ProgramResult {
        self.global_state.admin_account = *self.new_admin_account.key;
        Ok(())
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct UpdateReferral<'info> {
    // global state
    // address = Pubkey::from_str(GLOBAL_STATE_ADDRESS).unwrap(),
    #[account(
        has_one = admin_account,
    )]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // admin account
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    // referral state
    #[account(mut)]
    pub referral_state: ProgramAccount<'info, ReferralState>,
}
impl<'info> UpdateReferral<'info> {
    pub fn process(
        &mut self,
        transfer_duration: u32,
        pause: bool,
        optional_new_partner_account: Option<Pubkey>,
        operation_deposit_sol_fee: Option<u8>,
        operation_deposit_stake_account_fee: Option<u8>,
        operation_liquid_unstake_fee: Option<u8>,
    ) -> ProgramResult {
        self.referral_state.transfer_duration = transfer_duration;
        self.referral_state.pause = pause;
        // change partner_account if sent
        if let Some(new_partner_account) = optional_new_partner_account {
            self.referral_state.partner_account = new_partner_account
        }
        if let Some(operation_deposit_sol_fee) = self.cap_fee(operation_deposit_sol_fee)? {
            self.referral_state.operation_deposit_sol_fee = operation_deposit_sol_fee;
        }
        if let Some(operation_deposit_stake_account_fee) =
            self.cap_fee(operation_deposit_stake_account_fee)?
        {
            self.referral_state.operation_deposit_stake_account_fee =
                operation_deposit_stake_account_fee;
        }
        if let Some(operation_liquid_unstake_fee) = self.cap_fee(operation_liquid_unstake_fee)? {
            self.referral_state.operation_liquid_unstake_fee = operation_liquid_unstake_fee;
        }

        Ok(())
    }

    fn cap_fee(&self, new_fee: Option<u8>) -> std::result::Result<Option<u8>, ReferralError> {
        if let Some(new_fee) = new_fee {
            // considering the fee is calculated as basis points
            if new_fee > self.referral_state.max_operation_fee {
                msg!(
                    "Operation fee value {} is over maximal permitted {}; in basis points",
                    new_fee,
                    self.referral_state.max_operation_fee
                );
                return Err(ReferralOperationFeeOverMax);
            }
        }
        Ok(new_fee)
    }
}
