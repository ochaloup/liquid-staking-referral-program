use crate::{associated_token, error::*, instructions::*};
use anchor_lang::prelude::*;

pub fn process_update_authority(ctx: Context<UpdateAuthority>) -> ProgramResult {
    // TODO: security middleware
    // * if the mSOL mint is actually the mSOL mint
    // * for beneficiary_account (mSOL), if it exists, if it is really owned by the token program and if the mint is correct
    // * check System, Token, and associated_token_program to see if they're correct

    // check authority
    if ctx
        .accounts
        .state
        .partner_account
        .ne(ctx.accounts.partner_account.key)
    {
        return Err(ReferralError::AccessDenied.into());
    }

    // create associated token account for partner
    if **ctx.accounts.new_beneficiary_account.lamports.borrow() == 0_u64 {
        associated_token::create(ctx.accounts.into_create_associated_token_account_ctx())?;
    }

    ctx.accounts.state.partner_account = *ctx.accounts.new_partner_account.key;
    ctx.accounts.state.beneficiary_account = *ctx.accounts.new_beneficiary_account.key;

    Ok(())
}
