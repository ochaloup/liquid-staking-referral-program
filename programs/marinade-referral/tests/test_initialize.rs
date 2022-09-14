// use crate::integration_test::Rent;
// use crate::helper::{program_test, IntegrationTest};
// use crate::program_test;

// pub async fn check_initialize(
//     input: &impl InitializeInput,
//     banks_client: &mut BanksClient,
//     expected: &Marinade,
// ) -> anyhow::Result<()> {
//     // Check reflection is the same as expected
//     assert_eq!(
//         &Marinade::read_from_test(banks_client, &input.state().pubkey(), vec![]).await?,
//         expected
//     );

//     // Read state again for checking fields not included to reflection
//     let state: State = AccountDeserialize::try_deserialize(
//         &mut banks_client
//             .get_account(input.state().pubkey())
//             .await
//             .unwrap()
//             .unwrap()
//             .data
//             .as_slice(),
//     )?;

//     assert_eq!(*state.stake_system.stake_list_address(), input.stake_list());
//     assert_eq!(
//         *state.validator_system.validator_list_address(),
//         input.validator_list()
//     );
//     assert_eq!(state.liq_pool.msol_leg, input.liq_pool_msol_leg());
//     let stake_list_len = banks_client
//         .get_account(input.stake_list())
//         .await
//         .unwrap()
//         .unwrap()
//         .data
//         .len();
//     let validator_list_len = banks_client
//         .get_account(input.validator_list())
//         .await
//         .unwrap()
//         .unwrap()
//         .data
//         .len();
//     assert_eq!(
//         state
//             .stake_system
//             .stake_list_capacity(stake_list_len)
//             .unwrap(),
//         input.max_stake_count()
//     );
//     assert_eq!(
//         state
//             .validator_system
//             .validator_list_capacity(validator_list_len)
//             .unwrap(),
//         input.max_validator_count()
//     );
//     Ok(())
// }

/*#[test(tokio::test)]
async fn test_initialization_without_seeds() -> anyhow::Result<()> {
    let (mut banks_client, payer, recent_blockhash) = crate::program_test().start().await;
    let fee_payer = Arc::new(payer);
    let rent = banks_client.get_rent().await?;
    // let clock: Clock =
    //     bincode::deserialize(&banks_client.get_account(clock::ID).await?.unwrap().data)?;

    use rand_chacha::rand_core::SeedableRng;
    let mut rng = ChaChaRng::from_seed([
        102, 46, 250, 122, 194, 179, 201, 43, 230, 4, 42, 246, 158, 90, 248, 237, 8, 61, 81, 114,
        227, 137, 83, 10, 40, 93, 233, 9, 35, 24, 77, 213,
    ]);
    let input = InitializeInputWithoutSeeds::random(&mut rng);
    //let expected = input.expected_reflection(&rent, &clock);

    let builder = TransactionBuilder::unlimited(fee_payer);

    let transaction = input
        .build(builder, &rent)
        .build_one_combined()
        .unwrap()
        .into_signed(recent_blockhash)?;

    banks_client.process_transaction(transaction).await?;

    // let state: State = AccountDeserialize::try_deserialize(
    //     &mut banks_client
    //         .get_account(input.with_seeds.state.pubkey())
    //         .await?
    //         .unwrap()
    //         .data
    //         .as_slice(),
    // )?;
    //check_initialize(&input, &mut banks_client, &expected).await?;
    Ok(())
}

#[test(tokio::test)]
async fn test_initialization_with_seeds() -> anyhow::Result<()> {
    let (mut banks_client, payer, recent_blockhash) = crate::program_test().start().await;
    let fee_payer = Arc::new(payer);
    let rent = banks_client.get_rent().await?;
    // let clock: Clock =
    //     bincode::deserialize(&banks_client.get_account(clock::ID).await?.unwrap().data)?;

    let mut rng = ChaChaRng::from_seed([
        165, 207, 247, 199, 219, 129, 37, 113, 150, 217, 64, 249, 26, 198, 236, 23, 14, 109, 38,
        112, 152, 203, 30, 106, 214, 229, 34, 192, 20, 141, 116, 234,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);
    //let expected = input.expected_reflection(&rent, &clock);

    let builder = TransactionBuilder::unlimited(fee_payer);
    let transaction = input
        .build(builder, &rent)
        .build_one_combined()
        .unwrap()
        .into_signed(recent_blockhash)?;

    banks_client.process_transaction(transaction).await.unwrap();

    // let state: State = AccountDeserialize::try_deserialize(
    //     &mut banks_client
    //         .get_account(input.state.pubkey())
    //         .await?
    //         .unwrap()
    //         .data
    //         .as_slice(),
    // )?;
    //check_initialize(&input, &mut banks_client, &expected).await?;
    Ok(())
}
*/

/*
#[test(tokio::test)]
async fn test_empty_reserve() -> anyhow::Result<()> {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;
    let fee_payer = Arc::new(payer);
    let rent = banks_client.get_rent().await?;

    let mut rng = ChaChaRng::from_seed([
        159, 223, 172, 160, 99, 98, 67, 97, 50, 252, 75, 173, 149, 169, 58, 142, 110, 68, 63, 166,
        118, 32, 251, 6, 195, 18, 123, 22, 164, 220, 39, 70,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);

    let builder = TransactionBuilder::unlimited(fee_payer);
    let mut builder = builder
        .initialize(input.state.clone(), CREATOR_AUTHORITY.clone())
        .unwrap();
    builder.create_msol_mint(input.msol_mint.clone(), &rent);
    builder.set_admin_authority(input.admin_authority.pubkey());
    builder.set_operational_sol_account(input.operational_sol_account);
    builder.use_validator_manager_authority(input.validator_manager_authority.pubkey());
    builder.create_treasury_msol_account(input.treasury_msol_authority());
    builder.set_reward_fee(input.reward_fee);
    builder.assume_reserve_initialized(); // <- error
    builder.create_lp_mint(input.lp_mint.clone(), &rent);
    builder.init_liq_pool_sol_leg(0, &rent)?;
    builder.create_stake_list_with_seed(input.max_stake_count, &rent);
    builder.create_validator_list_with_seed(input.max_validator_count, &rent);
    builder.create_liq_pool_msol_leg_with_seed(&rent);

    let transaction = builder
        .build(&rent)
        .build_one_combined()
        .unwrap()
        .into_signed(recent_blockhash)?;

    assert_eq!(
        banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(11, InstructionError::InvalidArgument)
    );
    Ok(())
}
*/

/*
#[test(tokio::test)]
async fn test_fake_mint() -> anyhow::Result<()> {
    let mut test = program_test();
    let mut rng = ChaChaRng::from_seed([
        48, 200, 13, 148, 221, 72, 28, 60, 241, 255, 51, 45, 19, 65, 135, 178, 209, 153, 103, 95,
        240, 47, 73, 17, 175, 231, 216, 145, 187, 222, 130, 183,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);

    let mut fake_mint_account =
        Account::new(10000000, spl_token::state::Mint::LEN, &Pubkey::new_unique());
    let mint_state = spl_token::state::Mint {
        mint_authority: COption::Some(State::find_msol_mint_authority(&input.state.pubkey()).0),
        supply: 0,
        decimals: 9,
        is_initialized: true,
        freeze_authority: COption::None,
    };
    mint_state.pack_into_slice(&mut fake_mint_account.data);
    test.add_account(input.msol_mint.pubkey(), fake_mint_account);

    let (mut banks_client, payer, recent_blockhash) = test.start().await;
    let rent = banks_client.get_rent().await.unwrap();
    let fee_payer = Arc::new(payer);

    let builder = TransactionBuilder::unlimited(fee_payer);
    let mut builder = builder
        .initialize(input.state.clone(), CREATOR_AUTHORITY.clone())
        .unwrap();
    builder.use_msol_mint_pubkey(input.msol_mint.pubkey());
    builder.set_admin_authority(input.admin_authority.pubkey());
    builder.set_operational_sol_account(input.operational_sol_account);
    builder.use_validator_manager_authority(input.validator_manager_authority.pubkey());
    builder.create_treasury_msol_account(input.treasury_msol_authority());
    builder.set_reward_fee(input.reward_fee);
    builder.init_reserve(0, &rent)?;
    builder.create_lp_mint(input.lp_mint.clone(), &rent);
    builder.init_liq_pool_sol_leg(0, &rent)?;
    builder.create_stake_list_with_seed(input.max_stake_count, &rent);
    builder.create_validator_list_with_seed(input.max_validator_count, &rent);
    builder.create_liq_pool_msol_leg_with_seed(&rent);

    let transaction = builder
        .build(&rent)
        .build_one_combined()
        .unwrap()
        .into_signed(recent_blockhash)?;
    assert_eq!(
        banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(10, InstructionError::InvalidArgument)
    );
    Ok(())
}
*/
