// External deps
use crypto_exports::franklin_crypto::bellman::pairing::bn256::Bn256;
use num::BigUint;
// Workspace deps
use models::node::{operations::FullExitOp, FullExit};
// Local deps
use crate::witness::{
    full_exit::FullExitWitness,
    tests::test_utils::{generic_test_scenario, incorrect_op_test_scenario, WitnessTestAccount},
};

/// Checks that `FullExit` can be applied to an existing account.
/// Here we generate a PlasmaState with one account (which has some funds), and
/// apply a `FullExit` to this account.
#[test]
#[ignore]
fn test_full_exit_success() {
    // Input data.
    let accounts = vec![WitnessTestAccount::new(1, 10)];
    let account = &accounts[0];
    let full_exit_op = FullExitOp {
        priority_op: FullExit {
            account_id: account.id,
            eth_address: account.account.address,
            token: 0,
        },
        withdraw_amount: Some(BigUint::from(10u32).into()),
    };
    let success = true;

    generic_test_scenario::<FullExitWitness<Bn256>, _>(
        &accounts,
        (full_exit_op, success),
        (),
        |plasma_state, op| {
            plasma_state.apply_full_exit_op(&op.0);
            vec![]
        },
    );
}

#[test]
#[ignore]
fn test_full_exit_failure_no_account_in_tree() {
    // Input data.
    let accounts = &[];
    let account = WitnessTestAccount::new_empty(1); // Will not be included into PlasmaState
    let full_exit_op = FullExitOp {
        priority_op: FullExit {
            account_id: account.id,
            eth_address: account.account.address,
            token: 0,
        },
        withdraw_amount: None,
    };
    let success = false;

    generic_test_scenario::<FullExitWitness<Bn256>, _>(
        accounts,
        (full_exit_op, success),
        (),
        |plasma_state, op| {
            plasma_state.apply_full_exit_op(&op.0);
            vec![]
        },
    );
}

#[test]
#[ignore]
fn test_full_exit_initialted_from_wrong_account_owner() {
    // Input data.
    let accounts = vec![WitnessTestAccount::new(1, 10)];
    let invalid_account = WitnessTestAccount::new(2, 10);
    let account = &accounts[0];
    let invalid_account_eth_address = invalid_account.account.address;
    assert!(invalid_account_eth_address != account.account.address);
    let full_exit_op = FullExitOp {
        priority_op: FullExit {
            account_id: account.id,
            eth_address: invalid_account_eth_address,
            token: 0,
        },
        withdraw_amount: Some(BigUint::from(0u32).into()),
    };
    let success = false;

    generic_test_scenario::<FullExitWitness<Bn256>, _>(
        &accounts,
        (full_exit_op, success),
        (),
        |_plasma_state, _op| {
            // this operation should change nothing
            vec![]
        },
    );
}

/// Checks that executing a withdraw operation with incorrect
/// withdraw amount results in an error.
#[test]
#[ignore]
fn test_incorrect_full_exit_withdraw_amount() {
    // Test vector of (initial_balance, withdraw_amount, success).
    // Transactions are expected to fail with any value of provided `success` flag.
    let test_vector = vec![
        (10u64, 10000u64, true), // Withdraw too big and `success` set to true
        (0, 1, true),            // Withdraw from 0 balance and `success` set to true
        (10, 10000, false),      // Withdraw too big and `success` set to false
        (0, 1, false),           // Withdraw from 0 balance and `success` set to false
    ];

    // Operation is incorrect, since we try to withdraw more funds than account has.
    const ERR_MSG: &str = "op_valid is true/enforce equal to one";

    for (initial_balance, withdraw_amount, success) in test_vector {
        // Input data.
        let accounts = vec![WitnessTestAccount::new(1, initial_balance)];
        let account = &accounts[0];
        let full_exit_op = FullExitOp {
            priority_op: FullExit {
                account_id: account.id,
                eth_address: account.account.address,
                token: 0,
            },
            withdraw_amount: Some(BigUint::from(withdraw_amount).into()),
        };

        incorrect_op_test_scenario::<FullExitWitness<Bn256>, _>(
            &accounts,
            (full_exit_op, success),
            (),
            ERR_MSG,
            || vec![],
        );
    }
}
