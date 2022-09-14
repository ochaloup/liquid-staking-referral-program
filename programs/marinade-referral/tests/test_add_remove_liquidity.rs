//
// Integration Test
// add & remove liquidity
//

mod common;

use common::{initialize::InitializeInputWithSeeds, integration_test::*};
use std::sync::Arc;

use marinade_finance_offchain_sdk::{
    marinade_finance::State,
    spl_associated_token_account::get_associated_token_address,
};

use rand::{distributions::Uniform, prelude::Distribution, CryptoRng, RngCore, SeedableRng};
use rand_chacha::ChaChaRng;

use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use test_log::test;

pub struct AddLiquidityParams {
    pub user_sol: Arc<Keypair>,
    pub user_sol_balance: u64,
    pub added_liquidity: u64,
}

impl AddLiquidityParams {
    pub fn random<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        let user_sol_balance =
            Uniform::from((1 * LAMPORTS_PER_SOL)..(10 * LAMPORTS_PER_SOL)).sample(rng);
        Self {
            user_sol: Arc::new(Keypair::generate(rng)),
            user_sol_balance,
            added_liquidity: Uniform::from((LAMPORTS_PER_SOL / 2)..user_sol_balance).sample(rng),
        }
    }

    pub fn user_msol(&self, state: &State) -> Pubkey {
        get_associated_token_address(&self.user_sol.pubkey(), &state.msol_mint)
    }

    pub fn user_lp(&self, state: &State) -> Pubkey {
        get_associated_token_address(&self.user_sol.pubkey(), &state.liq_pool.lp_mint)
    }
}

#[test(tokio::test)]
async fn test_add_liquidity() -> anyhow::Result<()> {
    let mut rng = ChaChaRng::from_seed([
        248, 3, 94, 241, 228, 239, 32, 168, 219, 67, 27, 194, 26, 155, 140, 136, 154, 4, 40, 175,
        132, 80, 60, 31, 135, 250, 230, 19, 172, 106, 254, 120,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);
    let mut test = IntegrationTest::start(&input).await?;

    let mut alice = test
        .create_test_user("alice", 1001 * LAMPORTS_PER_SOL)
        .await;

    do_add_liquidity(&mut alice, random_amount(1, 1000, &mut rng), &mut test)
        .await
        .unwrap();
    Ok(())
}

#[test(tokio::test)]
async fn test_remove_all_liquidity() -> anyhow::Result<()> {
    let mut rng = ChaChaRng::from_seed([
        12, 186, 30, 97, 156, 49, 187, 56, 52, 208, 201, 14, 251, 244, 83, 79, 23, 190, 234, 108,
        198, 232, 147, 111, 207, 188, 128, 153, 82, 236, 69, 88,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);
    let mut test = IntegrationTest::start(&input).await?;
    let mut user = test
        .create_test_user("alice", 1001 * LAMPORTS_PER_SOL)
        .await;
    let liquidity_amount = random_amount(1, 1000, &mut rng);
    do_add_liquidity(&mut user, liquidity_amount, &mut test)
        .await
        .unwrap();
    do_remove_liquidity(&mut user, liquidity_amount, &mut test).await;
    Ok(())
}
