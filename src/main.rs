use std::{
    cmp::min,
    env,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use bitcoin::{
    key::{rand, rand::rngs::OsRng, Keypair},
    secp256k1, Address, Amount, Network, PrivateKey, XOnlyPublicKey,
};
use rand::Rng;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{
    types::{Payload, Root},
    utils::get_output_value_for_commit,
    worker::predicate,
};

#[cfg(test)]
mod test;
mod types;
mod utils;
mod worker;

const OUTPUT_BYTES_BASE: u64 = 43;
const DUST_AMOUNT: u64 = 546;

#[derive(Debug, Serialize, Deserialize)]
struct Success {
    sequence: u64,
    nonce: u64,
    time: u64,
    magic: String,
}

fn main() -> Result<()> {
    let arg = env::args().nth(1).unwrap();
    let msg = serde_json::from_str::<Root>(&arg).unwrap();

    let found = Arc::new(AtomicBool::new(false));
    let result_sequence = Arc::new(AtomicU64::new(0));
    let result_nonce = Arc::new(AtomicU64::new(0));
    let result_time = Arc::new(AtomicU64::new(0));
    rayon::scope(|ctx| {
        let seq_range_per_worker = worker::MAX_SEQUENCE / msg.concurrency;
        for i in 0..msg.concurrency {
            let seq_start = i * seq_range_per_worker;
            let mut seq_end = seq_start + seq_range_per_worker - 1;
            if i == msg.concurrency - 1 {
                seq_end = worker::MAX_SEQUENCE - 1;
            }
            let found_clone = Arc::clone(&found);
            let result_sequence_clone = Arc::clone(&result_sequence);
            let result_nonce_clone = Arc::clone(&result_nonce);
            let result_time_clone = Arc::clone(&result_time);

            let payload = get_payload(msg.clone(), None, None).unwrap();

            ctx.spawn(move |_s| {
                (seq_start..=seq_end).into_par_iter().find_any(|seq| {
                    if found_clone.load(Ordering::SeqCst) {
                        return false;
                    }
                    if seq % 10000 == 0 {
                        println!(
                            "Started mining for sequence: {} - {}",
                            seq,
                            min(seq + 10000, seq_end)
                        );
                    }
                    let res = predicate(*seq, &payload);
                    if res.is_err() {
                        println!("Error: {:#?}", res.err().unwrap());
                        return false;
                    }
                    if res.unwrap() {
                        found_clone.store(true, Ordering::SeqCst);
                        result_sequence_clone.store(*seq as u64, Ordering::SeqCst);
                        result_nonce_clone.store(payload.copied_data.args.nonce, Ordering::SeqCst);
                        result_time_clone.store(payload.copied_data.args.time, Ordering::SeqCst);
                        return true;
                    }
                    false
                });
            })
        }
    });

    let success = Success {
        sequence: result_sequence.load(Ordering::SeqCst),
        nonce: result_nonce.load(Ordering::SeqCst),
        time: result_time.load(Ordering::SeqCst),
        magic: "a87c1c7c-02a2-4d7d-ae59-81b176127c81".to_string(),
    };
    println!("{}", serde_json::to_string(&success).unwrap());
    Ok(())
}

fn get_payload(mut msg: Root, test_time: Option<u64>, test_nonce: Option<u64>) -> Result<Payload> {
    msg.copied_data.args.nonce = OsRng.gen_range(0..10000000);
    msg.copied_data.args.time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    if let Some(time) = test_time {
        msg.copied_data.args.time = time;
    }
    if let Some(nonce) = test_nonce {
        msg.copied_data.args.nonce = nonce;
    }
    let private_key = PrivateKey::from_wif(&msg.funding_wif)?;
    let secp = secp256k1::Secp256k1::new();
    let (xonly_pubkey, parity) =
        XOnlyPublicKey::from_keypair(&Keypair::from_secret_key(&secp, &private_key.inner));
    // get public key
    xonly_pubkey.public_key(parity);
    let (_address, fixed_output_script_pubkey) = utils::get_address_by_copied_data(
        &secp,
        &xonly_pubkey,
        &msg.copied_data,
        &msg.worker_options.op_type,
    );

    let private_address = Address::p2tr(&secp, xonly_pubkey, None, Network::Bitcoin);

    let total_inputs_value = msg.funding_utxo.value;
    let total_outputs_value = utils::get_output_value_for_commit(msg.fees);
    let calculated_fee = total_inputs_value - total_outputs_value;
    let mut need_change_fee_output = false;
    let expected_fee = msg.fees.commit_fee_only + msg.worker_options.satsbyte * OUTPUT_BYTES_BASE;
    let difference_between_calculated_and_expected_fee = calculated_fee - expected_fee;
    if calculated_fee > 0
        && difference_between_calculated_and_expected_fee > 0
        && difference_between_calculated_and_expected_fee >= DUST_AMOUNT
    {
        need_change_fee_output = true;
    }

    Ok(Payload {
        secp,
        copied_data: msg.copied_data,
        funding_utxo_id: msg.funding_utxo.txid.parse()?,
        funding_utxo_index: msg.funding_utxo.index,
        funding_utxo_vout: msg.funding_utxo.vout,
        funding_utxo_value: Amount::from_sat(msg.funding_utxo.value),
        xonly_pub_key: xonly_pubkey,
        funding_private_key: private_key,
        funding_private_script_pubkey: private_address.script_pubkey(),
        funding_value: Amount::from_sat(difference_between_calculated_and_expected_fee),
        fixed_output_script_pubkey,
        fixed_output_value: Amount::from_sat(get_output_value_for_commit(msg.fees)),
        need_change_fee_output,
    })
}
