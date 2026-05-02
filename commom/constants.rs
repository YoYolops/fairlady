// Fair lady fs interaction paths
pub const DATA_FOLDER_PATH: &str = "./data"; // Relative to where the binary is ran
pub const SYSTEM_DATA_FOLDER_PATH: &str = "./.fairlady";
pub const SYSTEM_FOREIGN_DATA_PATH: &str = "./.fairlady/foreign";
pub const SYSTEM_DATABASE_PATH: &str = "./.fairlady/fairlady.db";

// fairlady behavior params
pub const USERDATA_UPDATE_TIME_SECONDS: u64 = 10; // blocks data updates for ten seconds after any previous on
pub const WATCHER_REACTION_TIME_SECONDS: u64 = 1; // After an event is detected of user's data folder, how many time to wait before uploading it

// IPFS related
pub const KUBO_RPC_BASE_URL: &str = "http://kubo_node:5001/api/v0";
pub const KUBO_DEFAULT_MFS_DESTINATION_PATH: &str = "data.bin"; // This is the file name sent to kubo by fairlady
