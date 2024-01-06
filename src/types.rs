use bitcoin::{key::Secp256k1, secp256k1, Amount, PrivateKey, ScriptBuf, Txid, XOnlyPublicKey};
use minicbor::{data::Int, Encoder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Payload {
    pub copied_data: CopiedData,
    pub secp: Secp256k1<secp256k1::All>,
    pub funding_utxo_id: Txid,
    pub funding_utxo_index: u32,
    pub funding_utxo_vout: u32,
    pub funding_utxo_value: Amount,
    pub xonly_pub_key: XOnlyPublicKey,
    pub funding_private_key: PrivateKey,
    pub funding_private_script_pubkey: ScriptBuf,
    pub funding_value: Amount,
    pub fixed_output_script_pubkey: ScriptBuf,
    pub fixed_output_value: Amount,
    pub need_change_fee_output: bool,
    pub valid_prefix: Option<String>,
    pub valid_ext: Option<u8>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub copied_data: CopiedData,
    pub worker_options: WorkerOptions,
    #[serde(rename = "fundingWIF")]
    pub funding_wif: String,
    pub funding_utxo: FundingUtxo,
    pub fees: Fees,
    pub perform_bitwork_for_commit_tx: bool,
    pub worker_bitwork_info_commit: WorkerBitworkInfoCommit,
    pub concurrency: u32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopiedData {
    pub args: Args,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Args {
    pub time: u64,
    pub nonce: u64,
    pub bitworkc: String,
    pub bitworkr: String,
    #[serde(rename = "mint_ticker")]
    pub mint_ticker: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerOptions {
    pub electrum_api: ElectrumApi,
    pub satsbyte: u64,
    pub address: String,
    pub op_type: String,
    pub dmt_options: DmtOptions,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElectrumApi {
    pub base_url: String,
    pub use_post: bool,
    pub is_open_flag: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DmtOptions {
    pub mint_amount: i64,
    pub ticker: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingUtxo {
    pub txid: String,
    pub tx_id: String,
    pub output_index: u64,
    pub index: u32,
    pub vout: u32,
    pub value: u64,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fees {
    pub commit_and_reveal_fee: u64,
    pub commit_and_reveal_fee_plus_outputs: u64,
    pub reveal_fee_plus_outputs: u64,
    pub commit_fee_only: u64,
    pub reveal_fee_only: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerBitworkInfoCommit {
    #[serde(rename = "input_bitwork")]
    pub input_bitwork: String,
    #[serde(rename = "hex_bitwork")]
    pub hex_bitwork: String,
    pub prefix: Option<String>,
    pub ext: Option<u8>,
}

impl CopiedData {
    pub fn encode(&self) -> Vec<u8> {
        let buf = vec![];
        let mut encoder = Encoder::new(buf);
        encoder
            .map(1)
            .unwrap()
            .str("args")
            .unwrap()
            .map(5)
            .unwrap()
            .str("time")
            .unwrap()
            .int(Int::from(self.args.time))
            .unwrap()
            .str("nonce")
            .unwrap()
            .int(Int::from(self.args.nonce))
            .unwrap()
            .str("bitworkc")
            .unwrap()
            .str(self.args.bitworkc.as_str())
            .unwrap()
            .str("bitworkr")
            .unwrap()
            .str(self.args.bitworkr.as_str())
            .unwrap()
            .str("mint_ticker")
            .unwrap()
            .str(self.args.mint_ticker.as_str())
            .unwrap();
        encoder.into_writer()
    }
}
