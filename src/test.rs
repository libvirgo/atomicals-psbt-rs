use std::str::FromStr;

use crate::types::{Args, CopiedData};

#[test]
fn test_copied_data_encoded_ttts() {
    let copied_data = CopiedData {
        args: Args {
            time: 1704688101,
            nonce: 7588557,
            bitworkc: Some(String::from_str("000000").unwrap()),
            bitworkr: None,
            mint_ticker: "ttts".to_string(),
        },
    };
    assert_eq!(hex::encode(copied_data.encode()).as_str(), "a16461726773a46474696d651a659b79e5656e6f6e63651a0073cacd68626974776f726b63663030303030306b6d696e745f7469636b65726474747473");
}

#[test]
fn test_copied_data_encoded_voids() {
    let copied_data = CopiedData {
        args: Args {
            bitworkc: Some(String::from_str("0000000").unwrap()),
            bitworkr: Some(String::from_str("0000000").unwrap()),
            nonce: 4544581,
            time: 1704691417,
            mint_ticker: "voids".to_string(),
        },
    };
    assert_eq!(hex::encode(copied_data.encode()).as_str(), "a16461726773a56474696d651a659b86d9656e6f6e63651a0045584568626974776f726b63673030303030303068626974776f726b7267303030303030306b6d696e745f7469636b657265766f696473");
}
