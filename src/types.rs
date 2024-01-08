use bitcoin::{key::Secp256k1, secp256k1, Amount, PrivateKey, ScriptBuf, Txid, XOnlyPublicKey};
use minicbor::{data::Int, Encoder};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

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
    #[serde(default)]
    pub network: Network,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Network {
    #[default]
    Bitcoin,
    Test,
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
    pub bitworkc: Option<String>,
    pub bitworkr: Option<String>,
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
        let mut valid_args = 3;
        if self.args.bitworkc.is_some() {
            valid_args += 1;
        }
        if self.args.bitworkr.is_some() {
            valid_args += 1;
        }
        encoder
            .map(1)
            .unwrap()
            .str("args")
            .unwrap()
            .map(valid_args)
            .unwrap()
            .str("time")
            .unwrap()
            .int(Int::from(self.args.time))
            .unwrap()
            .str("nonce")
            .unwrap()
            .int(Int::from(self.args.nonce))
            .unwrap();
        if let Some(bitworkc) = &self.args.bitworkc {
            encoder
                .str("bitworkc")
                .unwrap()
                .str(bitworkc.as_str())
                .unwrap();
        }
        if let Some(bitworkr) = &self.args.bitworkr {
            encoder
                .str("bitworkr")
                .unwrap()
                .str(bitworkr.as_str())
                .unwrap();
        }
        encoder
            .str("mint_ticker")
            .unwrap()
            .str(self.args.mint_ticker.as_str())
            .unwrap();
        encoder.into_writer()
    }
}

impl From<Network> for bitcoin::Network {
    fn from(network: Network) -> Self {
        match network {
            Network::Bitcoin => bitcoin::Network::Bitcoin,
            Network::Test => bitcoin::Network::Testnet,
        }
    }
}
