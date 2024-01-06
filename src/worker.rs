use std::collections::{BTreeMap, HashMap};

use bitcoin::{
    absolute::LockTime,
    psbt::{Input, Output},
    sighash::{Prevouts, SighashCache},
    transaction::Version,
    OutPoint, Psbt, ScriptBuf, Sequence, TapSighashType, Transaction, TxIn, TxOut, Witness,
};

use crate::{types::Payload, utils};

pub(crate) const MAX_SEQUENCE: u32 = 0xFFFFFFFF;

pub(crate) fn predicate(seq: u32, payload: &Payload) -> anyhow::Result<bool> {
    let mut psbt = Psbt {
        unsigned_tx: Transaction {
            version: Version::ONE,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: payload.funding_utxo_id,
                    vout: payload.funding_utxo_vout,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence(seq), // Ignore nSequence.
                witness: Witness::default(),
            }],
            output: vec![TxOut {
                value: payload.fixed_output_value,
                script_pubkey: payload.fixed_output_script_pubkey.clone(),
            }],
        },
        version: 0,
        xpub: Default::default(),
        proprietary: Default::default(),
        unknown: Default::default(),
        inputs: vec![Input {
            witness_utxo: {
                Some(TxOut {
                    value: payload.funding_utxo_value,
                    script_pubkey: payload.funding_private_script_pubkey.clone(),
                })
            },
            tap_internal_key: Some(payload.xonly_pub_key),
            ..Default::default()
        }],
        outputs: vec![
            Output {
                ..Default::default()
            },
            Output {
                ..Default::default()
            },
        ],
    };
    if payload.need_change_fee_output {
        psbt.unsigned_tx.output.push(TxOut {
            value: payload.funding_value,
            script_pubkey: payload.funding_private_script_pubkey.clone(),
        });
    }
    let input_txouts = TxOut {
        value: payload.funding_utxo_value,
        script_pubkey: payload.funding_private_script_pubkey.clone(),
    };
    // SIGNER
    let unsigned_tx = psbt.unsigned_tx.clone();
    psbt.inputs
        .iter_mut()
        .enumerate()
        .try_for_each::<_, anyhow::Result<()>>(|(_vout, input)| {
            let hash_ty = input
                .sighash_type
                .and_then(|psbt_sighash_type| psbt_sighash_type.taproot_hash_ty().ok())
                .unwrap_or(TapSighashType::Default);
            let hash = SighashCache::new(&unsigned_tx).taproot_key_spend_signature_hash(
                0,
                &Prevouts::All(&[input_txouts.clone()]),
                hash_ty,
            )?;

            let secret_key = payload.funding_private_key.inner;
            utils::sign_psbt_taproot(
                &secret_key,
                input.tap_internal_key.unwrap(),
                None,
                input,
                hash,
                hash_ty,
                &payload.secp,
            );

            Ok(())
        })?;

    // FINALIZER
    psbt.inputs.iter_mut().for_each(|input| {
        let mut script_witness: Witness = Witness::new();
        script_witness.push(input.tap_key_sig.unwrap().to_vec());
        input.final_script_witness = Some(script_witness);

        // Clear all the data fields as per the spec.
        input.partial_sigs = BTreeMap::new();
        input.sighash_type = None;
        input.redeem_script = None;
        input.witness_script = None;
        input.bip32_derivation = BTreeMap::new();
    });

    // EXTRACTOR
    let tx = psbt.extract_tx_unchecked_fee_rate();
    if has_valid_bitwork(
        &tx.txid().to_string(),
        &payload.valid_prefix,
        &payload.valid_ext,
    ) {
        println!("Found sequence: {}", seq);
        println!("Txid: {}", tx.txid());
        return Ok(true);
    }
    Ok(false)
}

fn has_valid_bitwork(txid: &str, bitwork: &Option<String>, bitworkx: &Option<u8>) -> bool {
    if let Some(bitwork) = bitwork {
        if txid.starts_with(bitwork.as_str()) {
            if let Some(bitworkx_value) = bitworkx {
                let next_char = txid.chars().nth(bitwork.len());
                let mut char_map = HashMap::new();
                for (i, ch) in "0123456789abcdef".chars().enumerate() {
                    char_map.insert(ch, i as u8);
                }
                if let Some(next_char_value) = next_char.and_then(|ch| char_map.get(&ch)) {
                    if next_char_value >= bitworkx_value {
                        return true;
                    }
                }
            } else {
                return true;
            }
        }
    }
    false
}
