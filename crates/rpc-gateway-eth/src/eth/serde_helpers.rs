//! custom serde helper functions
use alloy_primitives::U256;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

pub mod sequence {
    use serde::{
        Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned, ser::SerializeSeq,
    };

    pub fn serialize<S, T>(val: &T, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        let mut seq = s.serialize_seq(Some(1))?;
        seq.serialize_element(val)?;
        seq.end()
    }

    pub fn deserialize<'de, T, D>(d: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: DeserializeOwned,
    {
        let mut seq = Vec::<T>::deserialize(d)?;
        if seq.len() != 1 {
            return Err(serde::de::Error::custom(format!(
                "expected params sequence with length 1 but got {}",
                seq.len()
            )));
        }
        Ok(seq.remove(0))
    }
}

/// A module that deserializes `[]` optionally
pub mod empty_params {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(d: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        let seq = Option::<Vec<()>>::deserialize(d)?.unwrap_or_default();
        if !seq.is_empty() {
            return Err(serde::de::Error::custom(format!(
                "expected params sequence with length 0 but got {}",
                seq.len()
            )));
        }
        Ok(())
    }

    pub fn serialize<S, T>(_val: &T, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        s.serialize_unit()
    }
}

/// A module that deserializes either a BlockNumberOrTag, or a simple number.
pub mod lenient_block_number {
    use alloy_rpc_types::BlockNumberOrTag;
    use serde::{Deserialize, Deserializer};

    /// Following the spec the block parameter is either:
    ///
    /// > HEX String - an integer block number
    /// > String "earliest" for the earliest/genesis block
    /// > String "latest" - for the latest mined block
    /// > String "pending" - for the pending state/transactions
    ///
    /// and with EIP-1898:
    /// > blockNumber: QUANTITY - a block number
    /// > blockHash: DATA - a block hash
    ///
    /// <https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1898.md>
    ///
    /// EIP-1898 does not all calls that use `BlockNumber` like `eth_getBlockByNumber` and doesn't
    /// list raw integers as supported.
    ///
    /// However, there are dev node implementations that support integers, such as ganache: <https://github.com/foundry-rs/foundry/issues/1868>
    ///
    /// N.B.: geth does not support ints in `eth_getBlockByNumber`
    pub fn lenient_block_number<'de, D>(deserializer: D) -> Result<BlockNumberOrTag, D::Error>
    where
        D: Deserializer<'de>,
    {
        LenientBlockNumber::deserialize(deserializer).map(Into::into)
    }

    /// Same as `lenient_block_number` but requires to be `[num; 1]`
    pub fn lenient_block_number_seq<'de, D>(deserializer: D) -> Result<BlockNumberOrTag, D::Error>
    where
        D: Deserializer<'de>,
    {
        let num = <[LenientBlockNumber; 1]>::deserialize(deserializer)?[0].into();
        Ok(num)
    }

    /// Various block number representations, See [`lenient_block_number()`]
    #[derive(Clone, Copy, Deserialize)]
    #[serde(untagged)]
    pub enum LenientBlockNumber {
        BlockNumber(BlockNumberOrTag),
        Num(u64),
    }

    impl From<LenientBlockNumber> for BlockNumberOrTag {
        fn from(b: LenientBlockNumber) -> Self {
            match b {
                LenientBlockNumber::BlockNumber(b) => b,
                LenientBlockNumber::Num(b) => b.into(),
            }
        }
    }
}

/// Helper type to parse both `u64` and `U256`
#[derive(Copy, Clone, Deserialize)]
#[serde(untagged)]
pub enum Numeric {
    /// A [U256] value.
    U256(U256),
    /// A `u64` value.
    Num(u64),
}

impl From<Numeric> for U256 {
    fn from(n: Numeric) -> Self {
        match n {
            Numeric::U256(n) => n,
            Numeric::Num(n) => Self::from(n),
        }
    }
}

impl FromStr for Numeric {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(val) = s.parse::<u128>() {
            Ok(Self::U256(U256::from(val)))
        } else if s.starts_with("0x") {
            U256::from_str_radix(s, 16)
                .map(Numeric::U256)
                .map_err(|err| err.to_string())
        } else {
            U256::from_str(s)
                .map(Numeric::U256)
                .map_err(|err| err.to_string())
        }
    }
}

/// Deserializes a number from hex or int
pub fn deserialize_number<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    Numeric::deserialize(deserializer).map(Into::into)
}
