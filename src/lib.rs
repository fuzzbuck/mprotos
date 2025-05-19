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
    use serde_with::serde_as;
    use solana_sdk::transaction::VersionedTransaction;
    use thiserror::Error;

    #[serde_as]
    #[derive(Clone, Serialize, Deserialize)]
    pub struct SerializedSignature {
        #[serde_as(as = "[_; 64]")]
        inner: [u8; 64]
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
}
