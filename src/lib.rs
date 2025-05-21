pub mod hook_proto {
    tonic::include_proto!("hook");
    tonic::include_proto!("vhook");
}

pub fn get_unix_epoch() -> u64 {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    duration.as_millis() as u64
}

pub mod vhook {
    use std::fmt::{Debug, Display, Formatter};
    use serde::{Deserialize, Serialize};
    use solana_sdk::signature::Signature;
    use solana_sdk::transaction::VersionedTransaction;
    use thiserror::Error;

    #[derive(Clone)]
    pub struct SerializedSignature {
        inner: [u8; 64]
    }
    
    impl From<&Signature> for SerializedSignature {
        fn from(value: &Signature) -> Self {
            Self {
                inner: *value.as_array()
            }
        }
    }

    impl serde::Serialize for SerializedSignature {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let encoded = bs58::encode(&self.inner).into_string();
            serializer.serialize_str(&encoded)
        }
    }

    impl<'de> serde::Deserialize<'de> for SerializedSignature {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            let decoded = bs58::decode(&s)
                .into_vec()
                .map_err(serde::de::Error::custom)?;
            if decoded.len() != 64 {
                return Err(serde::de::Error::custom("Invalid length for SerializedSignature"));
            }
            let mut inner = [0u8; 64];
            inner.copy_from_slice(&decoded);
            Ok(SerializedSignature { inner })
        }
    }
    
    impl Display for SerializedSignature {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", bs58::encode(&self.inner).into_string())
        }
    }
    
    impl Debug for SerializedSignature {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", bs58::encode(&self.inner).into_string())
        }
    }


    #[derive(Serialize, Debug, Clone)]
    pub struct VHookSubmitBundleRequest {
        pub uuid: String,
        pub auth_code: String,
        pub transactions: Vec<Vec<u8>>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct JitoBundle {
        pub bundle_id: String,
        pub transactions: Vec<VersionedTransaction>,
    }


    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VHookBundleStatus {
        pub bundle_id: String,
        pub executed_at: u64,
        pub serialized_error: Option<Vec<u8>>
    }
    
    impl VHookBundleStatus {
        pub fn try_get_error(&self) -> Option<RpcBundleExecutionError> {
            if let Some(serialized_error) = &self.serialized_error {
                bincode::deserialize::<RpcBundleExecutionError>(serialized_error).ok()
            } else {
                None
            }
        }
    }
    

    #[derive(Error, Debug, Clone, Serialize, Deserialize)]
    pub enum RpcBundleExecutionError {
        #[error("The bank has hit the max allotted time for processing transactions")]
        BankProcessingTimeLimitReached,

        #[error("Error locking bundle because a transaction is malformed")]
        BundleLockError,

        #[error("Bundle execution timed out")]
        BundleExecutionTimeout,

        #[error("The bundle exceeds the cost model")]
        ExceedsCostModel,

        #[error("Invalid pre or post accounts")]
        InvalidPreOrPostAccounts,

        #[error("PoH record error: {0}")]
        PohRecordError(String),

        #[error("Tip payment error: {0}")]
        TipError(String),

        #[error("A transaction in the bundle failed to execute: [signature={0}, error={1}]")]
        TransactionFailure(SerializedSignature, String),
    }

    #[derive(Serialize)]
    pub struct HookedBundle {
        pub uuid: String,
        pub auth_code: String,
        pub transactions: Vec<Vec<u8>>,
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json;
        use bs58;

        #[test]
        fn test_serialize_deserialize() {
            let data = [42u8; 64];
            let sig = SerializedSignature { inner: data };
            let json = serde_json::to_string(&sig).unwrap();
            println!("SerializedSignature base58: {}", json);
            let decoded: SerializedSignature = serde_json::from_str(&json).unwrap();
            assert_eq!(sig.inner, decoded.inner);
        }

        #[test]
        fn test_display_and_debug() {
            let data = [1u8; 64];
            let sig = SerializedSignature { inner: data };
            let encoded = bs58::encode(&data).into_string();
            assert_eq!(format!("{}", sig), encoded);
            assert_eq!(format!("{:?}", sig), encoded);
        }

        #[test]
        fn test_deserialize_invalid_length() {
            // 63 bytes, should fail
            let short = bs58::encode(&[0u8; 63]).into_string();
            let json = format!("\"{}\"", short);
            let result: Result<SerializedSignature, _> = serde_json::from_str(&json);
            assert!(result.is_err());
        }
    }
}
