use bitcoin::{
    hashes::Hash,
    key::{Keypair, Secp256k1, TapTweak},
    opcodes,
    psbt::Input,
    script::{Builder, PushBytesBuf},
    secp256k1, taproot,
    taproot::TaprootBuilder,
    Address, Network, ScriptBuf, TapLeafHash, TapSighash, TapSighashType, XOnlyPublicKey,
};

use crate::types::{CopiedData, Fees};

pub(crate) fn sign_psbt_taproot(
    secret_key: &secp256k1::SecretKey,
    pubkey: XOnlyPublicKey,
    leaf_hash: Option<TapLeafHash>,
    psbt_input: &mut Input,
    hash: TapSighash,
    hash_ty: TapSighashType,
    secp: &Secp256k1<secp256k1::All>,
) {
    let keypair = Keypair::from_seckey_slice(secp, secret_key.as_ref()).unwrap();
    let keypair = match leaf_hash {
        None => keypair
            .tap_tweak(secp, psbt_input.tap_merkle_root)
            .to_inner(),
        Some(_) => keypair, // no tweak for script spend
    };

    let msg = secp256k1::Message::from_digest(hash.to_byte_array());
    let sig = secp.sign_schnorr(&msg, &keypair);

    let final_signature = taproot::Signature { sig, hash_ty };

    if let Some(lh) = leaf_hash {
        psbt_input
            .tap_script_sigs
            .insert((pubkey, lh), final_signature);
    } else {
        psbt_input.tap_key_sig = Some(final_signature);
    }
}

pub(crate) fn get_output_value_for_commit(fees: Fees) -> u64 {
    fees.reveal_fee_plus_outputs
}

/// Returns the scriptPubkey for the commitment transaction output.
/// for print and test only
pub(crate) fn append_mint_update_reveal_script(
    keypair: &XOnlyPublicKey,
    payload: &CopiedData,
) -> String {
    let op_type = "dmt";
    let atomicals_protocol_envelope_id = "atom"; // replace with your actual value
    let mut ops = format!(
        "{} OP_CHECKSIG OP_0 OP_IF {} {}",
        hex::encode(keypair.serialize()),
        hex::encode(atomicals_protocol_envelope_id),
        hex::encode(op_type)
    );
    let cbor = payload.encode();
    for x in cbor.chunks(520) {
        ops += &format!(" {}", hex::encode(x));
    }

    ops += " OP_ENDIF";
    ops
}

pub(crate) fn get_address_by_copied_data(
    secp: &Secp256k1<secp256k1::All>,
    xonly_public_key: &XOnlyPublicKey,
    copied_data: &CopiedData,
    op_type: &String,
) -> (String, ScriptBuf) {
    let script =
        append_mint_update_reveal_script_by_builder(xonly_public_key, copied_data, op_type);
    let _str = append_mint_update_reveal_script(xonly_public_key, copied_data);
    let taproot_builder = TaprootBuilder::new();
    let resp = taproot_builder.add_leaf(0, script).unwrap();
    let spend_info = resp.finalize(secp, *xonly_public_key).unwrap();
    let addr = Address::p2tr_tweaked(spend_info.output_key(), Network::Bitcoin);
    (addr.to_string(), addr.script_pubkey())
}

fn append_mint_update_reveal_script_by_builder(
    xonly_public_key: &XOnlyPublicKey,
    payload: &CopiedData,
    optype: &String,
) -> ScriptBuf {
    let mut push_bytes_buf = PushBytesBuf::new();
    let atomicals_protocol_envelope_id = "atom"; // replace with your actual value
    let mut ops = Builder::new();
    ops = ops
        .push_x_only_key(xonly_public_key)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .push_opcode(opcodes::all::OP_PUSHBYTES_0)
        .push_opcode(opcodes::all::OP_IF);
    push_bytes_buf.clear();
    push_bytes_buf
        .extend_from_slice(atomicals_protocol_envelope_id.as_ref())
        .unwrap();
    ops = ops.push_slice(push_bytes_buf.as_push_bytes());
    push_bytes_buf.clear();
    push_bytes_buf.extend_from_slice(optype.as_ref()).unwrap();
    ops = ops.push_slice(push_bytes_buf.as_push_bytes());
    push_bytes_buf.clear();
    let cbor = payload.encode();
    for x in cbor.chunks(520) {
        push_bytes_buf.extend_from_slice(x).unwrap();
        ops = ops.push_slice(push_bytes_buf.as_push_bytes());
        push_bytes_buf.clear();
    }
    ops = ops.push_opcode(opcodes::all::OP_ENDIF);
    ops.into_script()
}
