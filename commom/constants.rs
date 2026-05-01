// Fair lady fs interaction paths
pub const DATA_FOLDER_PATH: &str = "./data"; // Relative to where the binary is ran
pub const SYSTEM_DATA_FOLDER_PATH: &str = "./.fairlady";
pub const SYSTEM_FOREIGN_DATA_PATH: &str = "./.fairlady/foreign";
pub const SYSTEM_DATABASE_PATH: &str = "./.fairlady/fairlady.db";
pub const USERDATA_UPDATE_TIME_SECONDS: u64 = 10;

// IPFS related
pub const KUBO_RPC_BASE_URL: &str = "http://kubo_node:5001/api/v0";
pub const KUBO_DEFAULT_MFS_DESTINATION_PATH: &str = "data.bin"; // This is the file name sent to kubo by fairlady
