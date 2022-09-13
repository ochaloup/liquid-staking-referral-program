use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::constant::*;
use crate::error::ReferralError::ReferralOperationFeeOverMax;
use crate::error::*;
use crate::states::{GlobalState, ReferralState};

use marinade_finance::{Fee};

//-----------------------------------------------------
#[derive(Accounts)]
pub struct Initialize<'info> {
    // admin account
    #[account(signer)]
    pub admin_account: AccountInfo<'info>,

    #[account(zero)]
    pub global_state: ProgramAccount<'info, GlobalState>,
}
impl<'info> Initialize<'info> {
    pub fn process(&mut self) -> ProgramResult {
        self.global_state.admin_account = self.admin_account.key();
        Ok(())
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct InitReferralAccount<'info> {
    // global state
    #[account(
        has_one = admin_account,
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
    pub msol_token_partner_account: CpiAccount<'info, TokenAccount>,
}

impl<'info> InitReferralAccount<'info> {
    pub fn process(&mut self, partner_name: String) -> ProgramResult {
        msg!("process_init_referral_account");
        if partner_name.len() > 20 {
            msg!("max partner_name.len() is 20");
            return Err(ReferralError::PartnerNameTooLong.into());
        }

        // check if beneficiary account address matches to partner_address and msol_mint
        if self.msol_token_partner_account.owner != *self.partner_account.key {
            return Err(ReferralError::InvalidBeneficiaryAccountOwner.into());
        }

        // verify the partner token account mint equals to treasury_msol_account
        if self.msol_token_partner_account.mint != self.treasury_msol_account.mint {
            return Err(ReferralError::InvalidBeneficiaryAccountMint.into());
        }

        self.referral_state.partner_name = partner_name.clone();

        self.referral_state.partner_account = self.partner_account.key();
        self.referral_state.msol_token_partner_account = self.msol_token_partner_account.key();

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

        self.referral_state.operation_deposit_sol_fee = Fee::from_basis_points(DEFAULT_OPERATION_FEE_POINTS);
        self.referral_state.operation_deposit_stake_account_fee = Fee::from_basis_points(DEFAULT_OPERATION_FEE_POINTS);
        self.referral_state.operation_liquid_unstake_fee = Fee::from_basis_points(DEFAULT_OPERATION_FEE_POINTS);
        self.referral_state.operation_delayed_unstake_fee = Fee::from_basis_points(DEFAULT_OPERATION_FEE_POINTS);

        Ok(())
    }
}

//--------------------------------------
#[derive(Accounts)]
pub struct ChangeAuthority<'info> {
    // global state
    #[account(mut, has_one = admin_account)]
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
        pause: bool,
        new_partner_account: Option<Pubkey>,
        operation_deposit_sol_fee: Option<u8>,
        operation_deposit_stake_account_fee: Option<u8>,
        operation_liquid_unstake_fee: Option<u8>,
        operation_delayed_unstake_fee: Option<u8>,
    ) -> ProgramResult {
        self.referral_state.pause = pause;

        // change partner_account if sent
        if let Some(new_partner_account) = new_partner_account {
            self.referral_state.partner_account = new_partner_account
        }

        self.referral_state.operation_deposit_sol_fee =
            self.checked_operation_fee(operation_deposit_sol_fee, self.referral_state.operation_deposit_sol_fee)?;
        self.referral_state.operation_deposit_stake_account_fee =
            self.checked_operation_fee(operation_deposit_stake_account_fee, self.referral_state.operation_deposit_stake_account_fee)?;
        self.referral_state.operation_liquid_unstake_fee =
            self.checked_operation_fee(operation_liquid_unstake_fee, self.referral_state.operation_liquid_unstake_fee)?;
        self.referral_state.operation_delayed_unstake_fee =
            self.checked_operation_fee(operation_delayed_unstake_fee, self.referral_state.operation_delayed_unstake_fee)?;

        Ok(())
    }

    fn checked_operation_fee(&self, new_fee: Option<u8>, default_value: Fee) -> std::result::Result<Fee, ReferralError> {
        if let Some(new_fee) = new_fee {
            // the fee is calculated as basis points
            if new_fee as u32 > MAX_OPERATION_FEE_POINTS {
                msg!(
                    "Operation fee value {} is over maximal permitted {}; in basis points",
                    new_fee,
                    MAX_OPERATION_FEE_POINTS
                );
                return Err(ReferralOperationFeeOverMax);
            }
            return Ok(Fee::from_basis_points(new_fee as u32));
        }
        Ok(default_value)
    }
}
