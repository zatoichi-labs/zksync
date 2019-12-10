// Built-in uses
use std::{env, path, fs, io, thread, time};
use std::sync::{Mutex, Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
// External uses
use ff::{Field, PrimeField};
use rand::Rng;
// Workspace uses
use prover;

#[test]
fn prover_sends_heartbeat_requests_and_exits_on_stop_signal() {
    // Testing [black box] that:
    // - Prover sends `working_on` requests (heartbeat) over api client
    // - Prover stops running upon receiving data over stop channel

    // Create a channel to notify on provers exit.
    let (done_tx, done_rx) = mpsc::channel();
    // Create channel to notify test about heartbeat requests.
    let (heartbeat_tx, heartbeat_rx) = mpsc::channel();

    // Run prover in a separate thread.
    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_ar = Arc::clone(&stop_signal);
    let circuit_parameters = read_circuit_parameters();
    let jubjub_params = franklin_crypto::alt_babyjubjub::AltJubjubBn256::new();
    thread::spawn(move || {
        // Create channel for proofs, not using in this test.
        let (tx, _) = mpsc::channel();
        let p =prover::Prover::new(
            circuit_parameters,
            jubjub_params,
            MockApiClient{
                block_to_prove: Mutex::new(Some(1)),
                heartbeats_tx: Arc::new(Mutex::new(heartbeat_tx)),
                publishes_tx: Arc::new(Mutex::new(tx)),
                prover_data_fn: || None,
            },
            time::Duration::from_millis(100),
            stop_signal_ar,
        );
        prover::start(p);
        println!("run exited!");
        done_tx.send(());
    });

    let timeout = time::Duration::from_millis(500);

    // Must receive heartbeat requests.
    heartbeat_rx.recv_timeout(timeout).expect("heartbeat request is not received");
    heartbeat_rx.recv_timeout(timeout).expect("heartbeat request is not received");

    // Send stop signal.
    println!("sending stop signal.");
    stop_signal.store(true, Ordering::SeqCst);
    heartbeat_rx.recv_timeout(timeout);
    heartbeat_rx.recv_timeout(timeout);
    println!("finishing up");
    // Prover must be stopped.
    done_rx.recv_timeout(timeout).unwrap();
}

#[test]
fn prover_proves_a_block_and_publishes_result() {
    // Testing [black box] the actual proof calculation by mocking genesis and +1 block.
    let circuit_params = read_circuit_parameters();
    let verify_key = bellman::groth16::prepare_verifying_key(&circuit_params.vk);
    let stop_signal = Arc::new(AtomicBool::new(false));
    let (proof_tx, proof_rx) = mpsc::channel();
    let (prover_data, public_data_commitment) = new_test_data_for_prover();

    // Run prover in separate thread.
    let stop_signal_ar = Arc::clone(&stop_signal);
    let jubjub_params = franklin_crypto::alt_babyjubjub::AltJubjubBn256::new();
    thread::spawn(move || {
        // Work heartbeat channel, not used in this test.
        let (tx, _) = mpsc::channel();
        let p = prover::Prover::new(
            circuit_params,
            jubjub_params,
            MockApiClient{
                block_to_prove: Mutex::new(Some(1)),
                heartbeats_tx: Arc::new(Mutex::new(tx)),
                publishes_tx: Arc::new(Mutex::new(proof_tx)),
                prover_data_fn: move || {
                    Some(prover_data.clone())
                },
            },
            time::Duration::from_secs(1),
            stop_signal_ar,
        );

        prover::start(p);
    });

    let timeout = time::Duration::from_secs(60*30); // 10 minutes
    let proof = proof_rx.recv_timeout(timeout).expect("didn't receive proof");
    stop_signal.store(true, Ordering::SeqCst);
    println!("verifying proof...");
    let verify_result = bellman::groth16::verify_proof(
        &verify_key,
        &proof.clone(),
        &[public_data_commitment]);
    assert!(!verify_result.is_err());
    assert!(verify_result.unwrap(), "invalid proof");
}

/// TestAccount is an account with random generated keys and address.
struct TestAccount {
    pub private_key: franklin_crypto::eddsa::PrivateKey<pairing::bn256::Bn256>,
    pub public_key: franklin_crypto::eddsa::PublicKey<pairing::bn256::Bn256>,
    pub address: models::node::account::AccountAddress
}

impl TestAccount {
    pub fn new() -> Self {
        let rng = &mut rand::thread_rng();
        let p_g = franklin_crypto::alt_babyjubjub::FixedGenerators::SpendingKeyGenerator;
        let jubjub_params = &franklin_crypto::alt_babyjubjub::AltJubjubBn256::new();
        let private_key = franklin_crypto::eddsa::PrivateKey::<pairing::bn256::Bn256>(rng.gen());
        let public_key = franklin_crypto::eddsa::PublicKey::<pairing::bn256::Bn256>::from_private(
            &private_key,
            p_g,
            jubjub_params,
        );
        let address = models::node::account::AccountAddress::from_pubkey(public_key);
        let public_key = franklin_crypto::eddsa::PublicKey::<pairing::bn256::Bn256>::from_private(
            &private_key,
            p_g,
            jubjub_params,
        );
        TestAccount{
            private_key,
            public_key,
            address,
        }
    }
}

fn new_test_data_for_prover() -> (prover::ProverData, models::node::Fr) {
    let mut circuit_tree = models::circuit::CircuitAccountTree::new(models::params::account_tree_depth() as u32);
    println!("Empty tree root hash: {}", circuit_tree.root_hash());

    let validator_test_account = TestAccount::new();
    println!("validator account address: {}", validator_test_account.address.to_hex());

    // Fee account
    let mut accounts = models::node::AccountMap::default();
    let mut validator_account = models::node::Account::default();
    validator_account.address = validator_test_account.address.clone();
    let validator_account_id: u32 = 0;
    accounts.insert(validator_account_id, validator_account.clone());

    let mut state = plasma::state::PlasmaState::new(accounts, 1);
    let genesis_root_hash = state.root_hash();
    println!("Genesis block root hash: {}", genesis_root_hash);
    circuit_tree.insert(0, models::circuit::account::CircuitAccount::from(validator_account.clone()));
    assert_eq!(circuit_tree.root_hash(), genesis_root_hash);

    let deposit_priority_op = models::node::FranklinPriorityOp::Deposit(
        models::node::Deposit{
            sender: web3::types::Address::zero(),
            token: 0,
            amount: bigdecimal::BigDecimal::from(10),
            account: validator_test_account.address.clone(),
        },
    );
    let mut op_success = state.execute_priority_op(deposit_priority_op.clone());
    let mut fees = Vec::new();
    let mut ops = Vec::new();
    let mut accounts_updated = Vec::new();

    if let Some(fee) = op_success.fee {
        fees.push(fee);
    }

    accounts_updated.append(&mut op_success.updates);

    ops.push(models::node::ExecutedOperations::PriorityOp(Box::new(models::node::ExecutedPriorityOp{
        op: op_success.executed_op,
        priority_op: models::node::PriorityOp{
            serial_id: 0,
            data: deposit_priority_op.clone(),
            deadline_block: 2,
            eth_fee: bigdecimal::BigDecimal::from(0),
            eth_hash: vec![0; 8],
        },
        block_index: 0,
    })));

    let (fee_account_id, fee_updates) = state.collect_fee(&fees, &validator_test_account.address);
    accounts_updated.extend(fee_updates.into_iter());

    let block = models::node::block::Block {
        block_number: state.block_number,
        new_root_hash: state.root_hash(),
        fee_account: fee_account_id,
        block_transactions: ops,
        processed_priority_ops: (0, 1),
    };
    println!("Block: {:?}", block);

    let mut pub_data = vec![];
    let mut operations = vec![];

    if let models::node::FranklinPriorityOp::Deposit(deposit_op) = deposit_priority_op {
        let deposit_witness = circuit::witness::deposit::apply_deposit_tx(&mut circuit_tree, &models::node::operations::DepositOp{
            priority_op: deposit_op,
            account_id: 0,
        });

        let deposit_operations = circuit::witness::deposit::calculate_deposit_operations_from_witness(
            &deposit_witness,
            &models::node::Fr::zero(),
            &models::node::Fr::zero(),
            &models::node::Fr::zero(),
            &circuit::operation::SignatureData{
                r_packed: vec![Some(false); 256],
                s: vec![Some(false); 256],
            },
            &[Some(false); 256],
        );
        operations.extend(deposit_operations);
        pub_data.extend(deposit_witness.get_pubdata());
    }

    let phaser = models::merkle_tree::PedersenHasher::<models::node::Engine>::default();
    let jubjub_params = &franklin_crypto::alt_babyjubjub::AltJubjubBn256::new();
    for _ in 0..models::params::block_size_chunks() - operations.len() {
        let (
            signature,
            first_sig_msg,
            second_sig_msg,
            third_sig_msg,
            _a,
            _b,
        ) = circuit::witness::utils::generate_dummy_sig_data(&[false], &phaser, &jubjub_params);

        operations.push(circuit::witness::noop::noop_operation(
            &circuit_tree,
            block.fee_account,
            &first_sig_msg,
            &second_sig_msg,
            &third_sig_msg,
            &signature,
            &[Some(false); 256],
        ));
        pub_data.extend(vec![false; 64]);
    }
    assert_eq!(pub_data.len(), 64 * models::params::block_size_chunks());
    assert_eq!(operations.len(), models::params::block_size_chunks());

    let validator_acc = circuit_tree
        .get(block.fee_account as u32)
        .expect("fee_account is not empty");
    let mut validator_balances = vec![];
    for i in 0..1 << models::params::BALANCE_TREE_DEPTH {
        let balance_value = match validator_acc.subtree.get(i as u32) {
            None => models::node::Fr::zero(),
            Some(bal) => bal.value,
        };
        validator_balances.push(Some(balance_value));
    }
    let _: models::node::Fr = circuit_tree.root_hash();
    let (root_after_fee, validator_account_witness) =
        circuit::witness::utils::apply_fee(&mut circuit_tree, block.fee_account, 0, 0);

    println!("root after fees {}", root_after_fee);
    println!("block new hash {}", block.new_root_hash);
    assert_eq!(root_after_fee, block.new_root_hash);
    let (validator_audit_path, _) =
        circuit::witness::utils::get_audits(&circuit_tree, block.fee_account as u32, 0);

    let public_data_commitment = circuit::witness::utils::public_data_commitment::<models::node::Engine>(
        &pub_data,
        Some(genesis_root_hash),
        Some(root_after_fee),
        Some(models::node::Fr::from_str(&block.fee_account.to_string()).unwrap()),
        Some(models::node::Fr::from_str(&(block.block_number).to_string()).unwrap()),
    );

    (prover::ProverData{
        public_data_commitment,
        old_root: genesis_root_hash,
        new_root: block.new_root_hash,
        validator_address: models::node::Fr::from_str(&block.fee_account.to_string()).unwrap(),
        operations,
        validator_balances,
        validator_audit_path,
        validator_account: validator_account_witness,
    }, public_data_commitment)
}

fn read_circuit_parameters() -> bellman::groth16::Parameters<models::node::Engine> {
    let out_dir = {
        let mut out_dir = path::PathBuf::new();
        out_dir.push(&env::var("KEY_DIR").expect("KEY_DIR not set"));
        out_dir.push(&format!("{}", models::params::block_size_chunks()));
        out_dir
    };
    let key_file_path = {
        let mut key_file_path = out_dir.clone();
        key_file_path.push(models::params::KEY_FILENAME);
        key_file_path
    };
    let f = fs::File::open(&key_file_path).expect("Unable to open file");
    let mut r = io::BufReader::new(f);
    bellman::groth16::Parameters::<models::node::Engine>::read(&mut r, true)
        .expect("Unable to read proving key")
}

struct MockApiClient<F: Fn() -> Option<prover::ProverData>> {
    block_to_prove: Mutex<Option<i64>>,
    heartbeats_tx: Arc<Mutex<mpsc::Sender<()>>>,
    publishes_tx: Arc<Mutex<mpsc::Sender<bellman::groth16::Proof<models::node::Engine>>>>,
    prover_data_fn: F,
}

impl<F: Fn() -> Option<prover::ProverData>> prover::ApiClient for MockApiClient<F> {
    fn block_to_prove(&self) -> Result<Option<i64>, String> {
        let block_to_prove = self.block_to_prove.lock().unwrap();
        Ok(*block_to_prove)
    }

    fn working_on(&self, block: i64) {
        let block_to_prove = self.block_to_prove.lock().unwrap();
        if let Some(stored) = *block_to_prove {
            if stored != block {
                return
            }
            self.heartbeats_tx.lock().unwrap().send(());
        }
    }

    fn prover_data(&self, block: i64) -> Result<prover::ProverData, String> {
        let block_to_prove = self.block_to_prove.lock().unwrap();
        if let Some(stored) = *block_to_prove {
            if stored != block {
                return Err("unexpected block".to_string())
            }
            let v = (self.prover_data_fn)();
            if let Some(pd) = v {
                return Ok(pd)
            }
        }
        Err("mock not configured".to_string())
    }

    fn publish(&self, p: bellman::groth16::Proof<models::node::Engine>) -> Result<(), String> {
        // No more blocks to prove. We're only testing single rounds.
        let mut block_to_prove = self.block_to_prove.lock().unwrap();
        *block_to_prove = None;

        self.publishes_tx.lock().unwrap().send(p);
        Ok(())
    }
}
