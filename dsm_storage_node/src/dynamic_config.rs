use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicConfig {
    pub dev_mode: bool,
    pub api_port: u16,
    pub storage_path: String,
    pub mpc_threshold: u32,
    pub mpc_participants: u32,
    pub storage_nodes: Vec<String>,
}

impl DynamicConfig {
    pub fn from_env() -> Self {
        let dev_mode = env::var("DEV_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        if dev_mode {
            Self::dev_config()
        } else {
            Self::prod_config()
        }
    }

    fn dev_config() -> Self {
        Self {
            dev_mode: true,
            api_port: 3001,
            storage_path: "./data".to_string(),
            mpc_threshold: 1,
            mpc_participants: 1,
            storage_nodes: vec!["http://127.0.0.1:3001".to_string()],
        }
    }

    fn prod_config() -> Self {
        Self {
            dev_mode: false,
            api_port: env::var("API_PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .unwrap_or(3001),
            storage_path: env::var("STORAGE_PATH").unwrap_or_else(|_| "./data".to_string()),
            mpc_threshold: env::var("DSM_MPC_THRESHOLD")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            mpc_participants: env::var("DSM_MPC_PARTICIPANTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            storage_nodes: env::var("DSM_STORAGE_NODES")
                .unwrap_or_else(|_| {
                    "http://127.0.0.1:3001,http://127.0.0.1:3002,http://127.0.0.1:3003".to_string()
                })
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        }
    }

    pub fn print_config(&self) {
        println!("🔧 DSM Storage Node Configuration:");
        if self.dev_mode {
            println!("   📱 Mode: DEVELOPMENT (Single Node)");
        } else {
            println!("   🏭 Mode: PRODUCTION (Multi Node)");
        }
        println!("   🔌 API Port: {}", self.api_port);
        println!("   💾 Storage Path: {}", self.storage_path);
        println!(
            "   🔐 MPC Threshold: {}/{}",
            self.mpc_threshold, self.mpc_participants
        );
        println!("   🌐 Storage Nodes: {:?}", self.storage_nodes);
        println!();
    }
}
