use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Comprehensive logging system for DSM Storage Node operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeOperationLog {
    pub id: String,
    pub timestamp: u64,
    pub node_id: String,
    pub operation_type: OperationType,
    pub details: OperationDetails,
    pub result: OperationResult,
    pub duration_ms: Option<u64>,
    pub peer_info: Option<PeerInfo>,
    pub client_info: Option<ClientInfo>,
    pub performance_metrics: Option<PerformanceMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    // Client interactions
    ClientConnection,
    ClientGenesis,
    ClientMpcContribution,
    ClientStorageRequest,
    ClientRetrieval,

    // Inter-node communication
    NodeDiscovery,
    NodeJoin,
    NodeHandshake,
    GossipSend,
    GossipReceive,
    ClusterSync,

    // MPC operations
    MpcSessionCreate,
    MpcContribution,
    MpcContributionReceive,
    MpcThresholdReached,
    MpcGenesisComplete,

    // Storage operations
    DataStore,
    DataRetrieve,
    DataDelete,
    DataSync,

    // Network operations
    NetworkConnect,
    NetworkDisconnect,
    NetworkError,
    HealthCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OperationDetails {
    pub description: String,
    pub data_size: Option<usize>,
    pub session_id: Option<String>,
    pub device_id: Option<String>,
    pub endpoint: Option<String>,
    pub error_message: Option<String>,
    pub retry_count: Option<u32>,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationResult {
    Success,
    Failure,
    Timeout,
    PartialSuccess,
    Retry,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeerInfo {
    pub peer_id: String,
    pub peer_endpoint: String,
    pub peer_region: Option<String>,
    pub connection_type: String, // "gossip", "bootstrap", "discovery"
    pub protocol_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientInfo {
    pub client_id: Option<String>,
    pub device_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_size: Option<usize>,
    pub response_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage_percent: Option<f64>,
    pub memory_usage_mb: Option<u64>,
    pub disk_usage_mb: Option<u64>,
    pub network_throughput_bps: Option<u64>,
    pub active_connections: Option<u32>,
    pub queue_size: Option<u32>,
}

/// Logger that tracks all storage node operations with detailed context
pub struct StorageNodeLogger {
    node_id: String,
    logs: tokio::sync::RwLock<Vec<NodeOperationLog>>,
    max_logs: usize,
}

impl StorageNodeLogger {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            logs: tokio::sync::RwLock::new(Vec::new()),
            max_logs: 10000, // Keep last 10k operations
        }
    }

    /// Log a storage node operation with full context
    pub async fn log_operation(
        &self,
        operation_type: OperationType,
        details: OperationDetails,
        result: OperationResult,
        duration_ms: Option<u64>,
        peer_info: Option<PeerInfo>,
        client_info: Option<ClientInfo>,
    ) {
        let log_entry = NodeOperationLog {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            node_id: self.node_id.clone(),
            operation_type: operation_type.clone(),
            details: details.clone(),
            result: result.clone(),
            duration_ms,
            peer_info: peer_info.clone(),
            client_info: client_info.clone(),
            performance_metrics: Self::collect_performance_metrics(),
        };

        // Log to console with appropriate level
        match result {
            OperationResult::Success => {
                info!(
                    "[{}] {:?} completed successfully in {}ms - {}",
                    self.node_id,
                    operation_type,
                    duration_ms.unwrap_or(0),
                    details.description
                );
            }
            OperationResult::Failure => {
                error!(
                    "[{}] {:?} failed - {} | Error: {}",
                    self.node_id,
                    operation_type,
                    details.description,
                    details.error_message.as_deref().unwrap_or("Unknown")
                );
            }
            OperationResult::Timeout => {
                warn!(
                    "[{}] {:?} timed out after {}ms - {}",
                    self.node_id,
                    operation_type,
                    duration_ms.unwrap_or(0),
                    details.description
                );
            }
            OperationResult::PartialSuccess => {
                warn!(
                    "[{}] {:?} partially successful - {}",
                    self.node_id, operation_type, details.description
                );
            }
            OperationResult::Retry => {
                debug!(
                    "[{}] {:?} retrying (attempt {}) - {}",
                    self.node_id,
                    operation_type,
                    details.retry_count.unwrap_or(0),
                    details.description
                );
            }
        }

        // Log additional context for specific operations
        if let Some(peer) = &peer_info {
            debug!(
                "[{}] Peer context: {} at {} ({})",
                self.node_id, peer.peer_id, peer.peer_endpoint, peer.connection_type
            );
        }

        if let Some(client) = &client_info {
            debug!(
                "[{}] Client context: device={} ip={} size={}B",
                self.node_id,
                client.device_id.as_deref().unwrap_or("unknown"),
                client.ip_address.as_deref().unwrap_or("unknown"),
                client.request_size.unwrap_or(0)
            );
        }

        // Store in memory log
        let mut logs = self.logs.write().await;
        logs.push(log_entry);

        // Trim logs if we exceed max size
        if logs.len() > self.max_logs {
            logs.remove(0);
        }
    }

    /// Get recent logs matching criteria
    pub async fn get_logs(
        &self,
        operation_type: Option<OperationType>,
        result: Option<OperationResult>,
        since_timestamp: Option<u64>,
        limit: Option<usize>,
    ) -> Vec<NodeOperationLog> {
        let logs = self.logs.read().await;
        let mut filtered: Vec<NodeOperationLog> = logs
            .iter()
            .filter(|log| {
                if let Some(op_type) = &operation_type {
                    if std::mem::discriminant(&log.operation_type)
                        != std::mem::discriminant(op_type)
                    {
                        return false;
                    }
                }
                if let Some(res) = &result {
                    if std::mem::discriminant(&log.result) != std::mem::discriminant(res) {
                        return false;
                    }
                }
                if let Some(since) = since_timestamp {
                    if log.timestamp < since {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Sort by timestamp (newest first)
        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        if let Some(limit) = limit {
            filtered.truncate(limit);
        }

        filtered
    }

    /// Get operation statistics
    pub async fn get_statistics(&self, since_hours: Option<u64>) -> OperationStatistics {
        let logs = self.logs.read().await;
        let since_timestamp = since_hours.map(|hours| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .saturating_sub(hours * 3600)
        });

        let relevant_logs: Vec<&NodeOperationLog> = logs
            .iter()
            .filter(|log| since_timestamp.is_none_or(|since| log.timestamp >= since))
            .collect();

        let total_operations = relevant_logs.len();
        let successful_operations = relevant_logs
            .iter()
            .filter(|log| matches!(log.result, OperationResult::Success))
            .count();
        let failed_operations = relevant_logs
            .iter()
            .filter(|log| matches!(log.result, OperationResult::Failure))
            .count();

        let avg_duration = if total_operations > 0 {
            relevant_logs
                .iter()
                .filter_map(|log| log.duration_ms)
                .sum::<u64>() as f64
                / total_operations as f64
        } else {
            0.0
        };

        let mut operation_counts = HashMap::new();
        for log in &relevant_logs {
            let op_name = format!("{:?}", log.operation_type);
            *operation_counts.entry(op_name).or_insert(0) += 1;
        }

        OperationStatistics {
            total_operations,
            successful_operations,
            failed_operations,
            success_rate: if total_operations > 0 {
                (successful_operations as f64 / total_operations as f64) * 100.0
            } else {
                0.0
            },
            average_duration_ms: avg_duration,
            operation_counts,
        }
    }

    /// Collect current performance metrics
    fn collect_performance_metrics() -> Option<PerformanceMetrics> {
        // In a real implementation, this would collect actual system metrics
        // For now, we'll return None to indicate metrics aren't available
        None
    }

    /// Export logs to JSON for external analysis
    pub async fn export_logs(&self, format: ExportFormat) -> Result<String, serde_json::Error> {
        let logs = self.logs.read().await;
        match format {
            ExportFormat::Json => serde_json::to_string_pretty(&*logs),
            ExportFormat::JsonCompact => serde_json::to_string(&*logs),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStatistics {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub success_rate: f64,
    pub average_duration_ms: f64,
    pub operation_counts: HashMap<String, u32>,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    JsonCompact,
}

/// Helper macros for easy logging
#[macro_export]
macro_rules! log_client_operation {
    ($logger:expr, $op_type:expr, $description:expr, $result:expr, $device_id:expr) => {
        $logger
            .log_operation(
                $op_type,
                $crate::logging::OperationDetails {
                    description: $description.to_string(),
                    device_id: Some($device_id.to_string()),
                    ..Default::default()
                },
                $result,
                None,
                None,
                Some($crate::logging::ClientInfo {
                    device_id: Some($device_id.to_string()),
                    ..Default::default()
                }),
            )
            .await;
    };
}

#[macro_export]
macro_rules! log_peer_operation {
    ($logger:expr, $op_type:expr, $description:expr, $result:expr, $peer_id:expr, $peer_endpoint:expr) => {
        $logger
            .log_operation(
                $op_type,
                $crate::logging::OperationDetails {
                    description: $description.to_string(),
                    endpoint: Some($peer_endpoint.to_string()),
                    ..Default::default()
                },
                $result,
                None,
                Some($crate::logging::PeerInfo {
                    peer_id: $peer_id.to_string(),
                    peer_endpoint: $peer_endpoint.to_string(),
                    connection_type: "gossip".to_string(),
                    peer_region: None,
                    protocol_version: None,
                }),
                None,
            )
            .await;
    };
}

// Default implementations
