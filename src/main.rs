use std::{
    cmp::min,
    env,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::SystemTime,
};

use anyhow::Result;
use bitcoin::{
    key::{rand, rand::rngs::OsRng, Keypair},
    secp256k1, Address, Amount, PrivateKey, XOnlyPublicKey,
};
use rand::Rng;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{
    types::{Payload, Root},
    utils::get_output_value_for_commit,
    worker::predicate,
};
use crate::types::{Args, CopiedData};
use crate::utils::get_address_by_copied_data;

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
    let secp = secp256k1::Secp256k1::new();
    // TODO change to wif
    let private_key = PrivateKey::from_wif("")?;
    let (xonly_pubkey, parity) =
        XOnlyPublicKey::from_keypair(&Keypair::from_secret_key(&secp, &private_key.inner));

    use std::sync::atomic::{AtomicUsize, Ordering};

    // TODO commit nonce
    let time = AtomicUsize::new(1704172809);
    let mut nonce = None;
    while time.load(Ordering::SeqCst) > 0 {
        nonce = (0..=10000000).into_par_iter().find_any(|i| {
            if i % 1000000 == 0 {
                println!(
                    "Started mining for nonce: {} - {}",
                    i,
                    min(i + 1000000, 10000000)
                );
            }
            let current_time = time.load(Ordering::SeqCst);
            let (addr, _) = get_address_by_copied_data(
                &secp,
                &xonly_pubkey,
                &CopiedData {
                    args: Args {
                        time: current_time as u64,
                        nonce: *i,
                        bitworkc: Some("000000".to_string()),
                        bitworkr: Some("6238".to_string()),
                        mint_ticker: "sophon".to_string(),
                    },
                },
                &String::from("dmt"),
            );
            // TODO change to middle wallet address
            if addr == "" {
                return true;
            }
            false
        });
        if nonce.is_none() {
            println!("time: {} not found nonces...", time.load(Ordering::SeqCst));
            time.fetch_sub(1, Ordering::SeqCst);
        } else {
            break;
        }
    }
    println!("nonce {:?} time: {}", nonce, time.load(Ordering::SeqCst));
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

    let private_address = Address::p2tr(&secp, xonly_pubkey, None, msg.network.into());

    let total_inputs_value = msg.funding_utxo.value;
    let total_outputs_value = get_output_value_for_commit(msg.fees);
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
        valid_prefix: msg.worker_bitwork_info_commit.prefix,
        valid_ext: msg.worker_bitwork_info_commit.ext,
    })
}
