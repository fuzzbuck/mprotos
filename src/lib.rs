pub mod hook_proto {
    tonic::include_proto!("hook");
    tonic::include_proto!("vhook");
}

pub mod vhook {
    use std::fmt::{Display, Formatter};
    use serde::{Deserialize, Serialize};
    use serde_with::serde_as;
    use thiserror::Error;

    #[serde_as]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SerializedSignature {
        #[serde_as(as = "[_; 64]")]
        inner: [u8; 64]
    }
    
    impl Display for SerializedSignature {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", bs58::encode(&self.inner).into_string())
        }
    }


    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VHookBundleStatus {
        bundle_uuid: String,
        signatures: Vec<SerializedSignature>,
        error: Option<RpcBundleExecutionError>
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
}
