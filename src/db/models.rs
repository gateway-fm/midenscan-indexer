use miden_protocol::{
    account::{AccountStorageMode, AccountType, StorageSlotType},
    note::NoteType,
};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DatabaseRef {
    pub block_commitment: Vec<u8>,
    pub block_number: i32,
}

#[derive(Debug, Clone)]
pub struct DatabaseBlock {
    pub block_commitment: Vec<u8>,
    pub block_number: u32,
    pub version: u32,
    pub timestamp: u32,
    pub chain_commitment: Vec<u8>,
    pub account_root: Vec<u8>,
    pub nullifier_root: Vec<u8>,
    pub note_root: Vec<u8>,
    pub tx_commitment: Vec<u8>,
    pub proof_commitment: Vec<u8>,
    pub sub_commitment: Vec<u8>,
    pub number_of_transactions: u32,
}

#[derive(Debug, Clone)]
pub struct DatabaseAccount {
    pub account_bech: String,
    pub account_id: Vec<u8>,
    pub account_id_prefix: Vec<u8>,

    pub account_type: Option<DatabaseMidenAccountType>,
    pub storage_mode: DatabaseMidenAccountStorageMode,
    pub code: Option<String>,
    pub code_size: u64,
    pub code_procedure_roots: Option<Vec<String>>,

    pub deployed_at_block_number: u32,
    pub deployed_at_timestamp: u32,
    pub deployed_at_updated_account_index: u32,
    // {deployed_at_block_number}_{deployed_at_updated_account_index}_{}
    pub deployed_at_internal_time: u128,
}

#[derive(Debug, Clone)]
pub struct DatabaseAccountUpdate {
    // {account_bech}_{block_commitment}
    pub account_update_id: String,

    pub account_bech: String,
    pub final_state_commitment: Vec<u8>,
    pub nonce_delta: Option<u64>,

    pub block_number: u32,
    pub timestamp: u32,
    pub updated_account_index: u32,
    // {block_number}_{updated_account_index}_{}
    pub internal_time: u128,
}

#[derive(Debug, Clone)]
pub struct DatabaseAccountVaultAsset {
    // {account_bech}_{faucet_id_prefix}
    pub account_vault_asset_id: String,
    pub account_bech: String,
    pub faucet_id_prefix: Vec<u8>,
    pub amount: i64,
}

#[derive(Debug, Clone)]
pub struct DatabaseAccountStorageSlot {
    // {account_bech}_{slot_index}
    pub account_storage_slot_id: String,

    pub account_bech: String,
    pub slot_index: String,
    pub value: Vec<u8>,
    pub account_storage_slot_type: DatabaseAccountStorageSlotType,
    pub last_updated_at_block_number: u32,
    pub last_updated_at_account_update_id: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseAccountStorageSlotMap {
    // {account_bech}_{slot_index}_{key}
    pub account_storage_slot_map_id: String,
    pub account_bech: String,
    pub slot_index: String,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub last_updated_at_block_number: u32,
    pub last_updated_at_account_update_id: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseTransaction {
    pub transaction_id: Vec<u8>,

    // {account_bech}_{block_commitment}
    pub account_update_id: String,
    pub account_bech: String,
    pub block_number: u32,
    pub timestamp: u32,

    pub transaction_index: u32,
    pub initial_state_commitment: Vec<u8>,
    pub final_state_commitment: Vec<u8>,

    // {block_number}_{transaction_index}_0
    pub internal_time: u128,
}

#[derive(Debug, Clone)]
pub struct DatabaseTransactionInputNote {
    // {transaction_id}_{nullifier}
    pub transaction_input_note_id: String,
    pub transaction_id: Vec<u8>,
    pub nullifier: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct DatabaseTransactionOutputNote {
    // {transaction_id}_{note_id}
    pub transaction_output_note_id: String,
    pub transaction_id: Vec<u8>,
    pub note_id: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct DatabaseNote {
    pub note_id: Vec<u8>,

    pub recipient: Option<Vec<u8>>,
    pub sender: String,
    pub note_type: DatabaseMidenNoteType,
    pub note_tag: u32,
    pub note_aux: u64,
    pub nullifier: Option<Vec<u8>>,
    pub script_code: Option<String>,
    pub inputs: Option<Vec<u64>>,
    pub is_network: bool,

    pub block_number: u32,
    pub timestamp: u32,
    pub batch_index: u16,
    pub note_index_in_batch: u16,
    pub leaf_index: u16,
    // {block_number}_{absolute_index}_{}
    pub internal_time: u128,
}

#[derive(Debug, Clone)]
pub struct DatabaseNoteAsset {
    // {note_id}_{faucet_id_prefix}
    pub note_asset_id: String,
    pub note_id: Vec<u8>,
    pub faucet_id_prefix: Vec<u8>,
    pub amount: u64,
}

#[derive(Debug, Clone)]
pub struct DatabaseNoteTag {
    pub note_tag: u32,
    pub number_of_notes: u32,
}

#[derive(Debug, Clone)]
pub struct DatabaseNullifier {
    pub nullifier: Vec<u8>,
    pub nullifier_index: u32,
    pub block_number: u32,
    pub timestamp: u32,
    // {block_number}_{nullifier_index}_{}
    pub internal_time: u128,
}

#[derive(Debug, Clone)]
pub struct DatabaseFungibleFaucetAccount {
    pub account_bech: String,
    pub faucet_id_prefix: Vec<u8>,

    pub symbol: Option<String>,
    pub decimals: Option<u8>,
    pub max_supply: Option<u64>,
}

#[derive(sqlx::Type, Debug, Clone)]
#[sqlx(type_name = "miden_account_type")]
pub enum DatabaseMidenAccountType {
    FungibleFaucet,
    NonFungibleFaucet,
    RegularAccountImmutableCode,
    RegularAccountUpdatableCode,
}

impl From<AccountType> for DatabaseMidenAccountType {
    fn from(account_type: AccountType) -> Self {
        match account_type {
            AccountType::FungibleFaucet => DatabaseMidenAccountType::FungibleFaucet,
            AccountType::NonFungibleFaucet => DatabaseMidenAccountType::NonFungibleFaucet,
            AccountType::RegularAccountImmutableCode => {
                DatabaseMidenAccountType::RegularAccountImmutableCode
            }
            AccountType::RegularAccountUpdatableCode => {
                DatabaseMidenAccountType::RegularAccountUpdatableCode
            }
        }
    }
}

#[derive(sqlx::Type, Debug, Clone)]
#[sqlx(type_name = "miden_account_storage_mode")]
pub enum DatabaseMidenAccountStorageMode {
    Public,
    Network,
    Private,
}

impl From<AccountStorageMode> for DatabaseMidenAccountStorageMode {
    fn from(account_storage_mode: AccountStorageMode) -> Self {
        match account_storage_mode {
            AccountStorageMode::Public => DatabaseMidenAccountStorageMode::Public,
            AccountStorageMode::Network => DatabaseMidenAccountStorageMode::Network,
            AccountStorageMode::Private => DatabaseMidenAccountStorageMode::Private,
        }
    }
}

#[derive(sqlx::Type, Debug, Clone)]
#[sqlx(type_name = "miden_note_type")]
pub enum DatabaseMidenNoteType {
    Private,
    Encrypted,
    Public,
}

impl From<NoteType> for DatabaseMidenNoteType {
    fn from(note_type: NoteType) -> Self {
        match note_type {
            NoteType::Private => DatabaseMidenNoteType::Private,
            NoteType::Public => DatabaseMidenNoteType::Public,
        }
    }
}

#[derive(sqlx::Type, Debug, Clone)]
#[sqlx(type_name = "miden_account_storage_slot_type")]
pub enum DatabaseAccountStorageSlotType {
    Value,
    Map,
}
impl From<StorageSlotType> for DatabaseAccountStorageSlotType {
    fn from(account_storage_slot_type: StorageSlotType) -> Self {
        match account_storage_slot_type {
            StorageSlotType::Value => DatabaseAccountStorageSlotType::Value,
            StorageSlotType::Map => DatabaseAccountStorageSlotType::Map,
        }
    }
}
