//! Storage Node Observability and Monitoring
//!
//! This module provides comprehensive logging and monitoring capabilities for DSM storage nodes,
//! including MPC ceremony tracking, inter-node communication monitoring, and performance metrics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// Represents different types of operations that can be logged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    /// Client requests to storage node
    ClientRequest {
        endpoint: String,
        method: String,
        client_ip: String,
    },
    /// Inter-node communication
    NodeCommunication {
        peer_node_id: String,
        operation: String,
        direction: CommunicationDirection,
    },
    /// MPC ceremony operations
    MpcOperation {
        session_id: String,
        operation: String,
        participant_count: usize,
    },
    /// Storage operations
    StorageOperation {
        operation: String,
        key: String,
        size_bytes: Option<usize>,
    },
    /// Network topology changes
    TopologyChange {
        event: String,
        affected_nodes: Vec<String>,
    },
}

/// Direction of communication between nodes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CommunicationDirection {
    Incoming,
    Outgoing,
}

/// Detailed log entry for storage node operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageNodeLogEntry {
    /// Unique identifier for this log entry
    pub id: String,
    /// Timestamp when the operation occurred
    pub timestamp: u64,
    /// Node ID that generated this log
    pub node_id: String,
    /// Type of operation
    pub operation_type: OperationType,
    /// Duration of the operation in milliseconds
    pub duration_ms: Option<u64>,
    /// Success/failure status
    pub success: bool,
    /// Additional details about the operation
    pub details: HashMap<String, String>,
    /// Error message if operation failed
    pub error: Option<String>,
}

/// Tracks ongoing MPC sessions with detailed state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpcSessionTracker {
    pub session_id: String,
    pub device_id: String,
    pub threshold: usize,
    pub participants: Vec<String>,
    pub contributions_received: usize,
    pub started_at: u64,
    pub last_activity: u64,
    pub status: MpcSessionStatus,
    pub timeline: Vec<MpcTimelineEvent>,
}

/// Status of an MPC session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MpcSessionStatus {
    Collecting,
    Aggregating,
    Complete,
    Failed,
    Expired,
}

/// Individual events in an MPC session timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpcTimelineEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub node_id: String,
    pub details: HashMap<String, String>,
}

/// Performance metrics for storage node operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePerformanceMetrics {
    pub node_id: String,
    pub uptime_seconds: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub storage_operations: u64,
    pub mpc_sessions_handled: u64,
    pub peer_connections: usize,
    pub data_stored_bytes: u64,
    pub last_updated: u64,
}

/// Parameters for client request logging
#[derive(Debug, Clone)]
pub struct ClientRequestParams {
    pub endpoint: String,
    pub method: String,
    pub client_ip: String,
    pub duration_ms: Option<u64>,
    pub success: bool,
    pub details: HashMap<String, String>,
    pub error: Option<String>,
}

/// Parameters for node communication logging
#[derive(Debug, Clone)]
pub struct NodeCommunicationParams {
    pub peer_node_id: String,
    pub operation: String,
    pub direction: CommunicationDirection,
    pub duration_ms: Option<u64>,
    pub success: bool,
    pub details: HashMap<String, String>,
    pub error: Option<String>,
}

/// Centralized logging and monitoring system for storage nodes
pub struct StorageNodeMonitor {
    node_id: String,
    logs: Arc<RwLock<Vec<StorageNodeLogEntry>>>,
    mpc_sessions: Arc<RwLock<HashMap<String, MpcSessionTracker>>>,
    performance_metrics: Arc<RwLock<NodePerformanceMetrics>>,
    start_time: SystemTime,
}

impl StorageNodeMonitor {
    /// Create a new monitoring instance for a storage node
    pub fn new(node_id: String) -> Self {
        let start_time = SystemTime::now();
        let performance_metrics = NodePerformanceMetrics {
            node_id: node_id.clone(),
            uptime_seconds: 0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time_ms: 0.0,
            storage_operations: 0,
            mpc_sessions_handled: 0,
            peer_connections: 0,
            data_stored_bytes: 0,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        Self {
            node_id,
            logs: Arc::new(RwLock::new(Vec::new())),
            mpc_sessions: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(performance_metrics)),
            start_time,
        }
    }

    /// Log a client request
    pub async fn log_client_request(&self, params: ClientRequestParams) {
        let entry = StorageNodeLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            node_id: self.node_id.clone(),
            operation_type: OperationType::ClientRequest {
                endpoint: params.endpoint.clone(),
                method: params.method.clone(),
                client_ip: params.client_ip.clone(),
            },
            duration_ms: params.duration_ms,
            success: params.success,
            details: params.details.clone(),
            error: params.error.clone(),
        };

        info!(
            node_id = %self.node_id,
            endpoint = %params.endpoint,
            method = %params.method,
            client_ip = %params.client_ip,
            duration_ms = ?params.duration_ms,
            success = %params.success,
            "Client request processed"
        );

        let mut logs = self.logs.write().await;
        logs.push(entry);

        // Update metrics
        self.update_request_metrics(params.success, params.duration_ms)
            .await;
    }

    /// Log inter-node communication
    pub async fn log_node_communication(&self, params: NodeCommunicationParams) {
        let entry = StorageNodeLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            node_id: self.node_id.clone(),
            operation_type: OperationType::NodeCommunication {
                peer_node_id: params.peer_node_id.clone(),
                operation: params.operation.clone(),
                direction: params.direction,
            },
            duration_ms: params.duration_ms,
            success: params.success,
            details: params.details.clone(),
            error: params.error.clone(),
        };

        info!(
            node_id = %self.node_id,
            peer_node_id = %params.peer_node_id,
            operation = %params.operation,
            direction = ?params.direction,
            duration_ms = ?params.duration_ms,
            success = %params.success,
            "Node communication logged"
        );

        let mut logs = self.logs.write().await;
        logs.push(entry);
    }

    /// Start tracking an MPC session
    pub async fn start_mpc_session_tracking(
        &self,
        session_id: &str,
        device_id: &str,
        threshold: usize,
    ) {
        let tracker = MpcSessionTracker {
            session_id: session_id.to_string(),
            device_id: device_id.to_string(),
            threshold,
            participants: Vec::new(),
            contributions_received: 0,
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            last_activity: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: MpcSessionStatus::Collecting,
            timeline: vec![MpcTimelineEvent {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                event_type: "session_started".to_string(),
                node_id: self.node_id.clone(),
                details: [
                    ("device_id".to_string(), device_id.to_string()),
                    ("threshold".to_string(), threshold.to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
            }],
        };

        info!(
            session_id = %session_id,
            device_id = %device_id,
            threshold = %threshold,
            "MPC session tracking started"
        );

        let mut sessions = self.mpc_sessions.write().await;
        sessions.insert(session_id.to_string(), tracker);

        // Update metrics
        let mut metrics = self.performance_metrics.write().await;
        metrics.mpc_sessions_handled += 1;
    }

    /// Log MPC contribution received
    pub async fn log_mpc_contribution(
        &self,
        session_id: &str,
        contributor_node_id: &str,
        contribution_size: usize,
        success: bool,
        error: Option<String>,
    ) {
        // Update session tracker
        let mut sessions = self.mpc_sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.participants.push(contributor_node_id.to_string());
            session.contributions_received += 1;
            session.last_activity = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let mut details = HashMap::new();
            details.insert("contributor".to_string(), contributor_node_id.to_string());
            details.insert(
                "contribution_size".to_string(),
                contribution_size.to_string(),
            );
            details.insert(
                "total_contributions".to_string(),
                session.contributions_received.to_string(),
            );
            details.insert("threshold".to_string(), session.threshold.to_string());

            if session.contributions_received >= session.threshold {
                session.status = MpcSessionStatus::Aggregating;
                details.insert("threshold_met".to_string(), "true".to_string());
            }

            session.timeline.push(MpcTimelineEvent {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                event_type: "contribution_received".to_string(),
                node_id: contributor_node_id.to_string(),
                details: details.clone(),
            });

            info!(
                session_id = %session_id,
                contributor = %contributor_node_id,
                contributions = %session.contributions_received,
                threshold = %session.threshold,
                threshold_met = %(session.contributions_received >= session.threshold),
                "MPC contribution received"
            );
        }

        // Log the operation
        let entry = StorageNodeLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            node_id: self.node_id.clone(),
            operation_type: OperationType::MpcOperation {
                session_id: session_id.to_string(),
                operation: "contribution_received".to_string(),
                participant_count: sessions
                    .get(session_id)
                    .map(|s| s.participants.len())
                    .unwrap_or(0),
            },
            duration_ms: None,
            success,
            details: [
                ("contributor".to_string(), contributor_node_id.to_string()),
                (
                    "contribution_size".to_string(),
                    contribution_size.to_string(),
                ),
            ]
            .iter()
            .cloned()
            .collect(),
            error,
        };

        let mut logs = self.logs.write().await;
        logs.push(entry);
    }

    /// Complete MPC session tracking
    pub async fn complete_mpc_session(
        &self,
        session_id: &str,
        success: bool,
        result_details: HashMap<String, String>,
    ) {
        let mut sessions = self.mpc_sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = if success {
                MpcSessionStatus::Complete
            } else {
                MpcSessionStatus::Failed
            };
            session.last_activity = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            session.timeline.push(MpcTimelineEvent {
                timestamp: session.last_activity,
                event_type: "session_completed".to_string(),
                node_id: self.node_id.clone(),
                details: result_details.clone(),
            });

            info!(
                session_id = %session_id,
                success = %success,
                participants = %session.participants.len(),
                duration_seconds = %(session.last_activity - session.started_at),
                "MPC session completed"
            );
        }
    }

    /// Get detailed MPC session information
    pub async fn get_mpc_session_details(&self, session_id: &str) -> Option<MpcSessionTracker> {
        let sessions = self.mpc_sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Get all active MPC sessions
    pub async fn get_active_mpc_sessions(&self) -> Vec<MpcSessionTracker> {
        let sessions = self.mpc_sessions.read().await;
        sessions
            .values()
            .filter(|s| {
                matches!(
                    s.status,
                    MpcSessionStatus::Collecting | MpcSessionStatus::Aggregating
                )
            })
            .cloned()
            .collect()
    }

    /// Log storage operation
    pub async fn log_storage_operation(
        &self,
        operation: &str,
        key: &str,
        size_bytes: Option<usize>,
        duration_ms: Option<u64>,
        success: bool,
        error: Option<String>,
    ) {
        let entry = StorageNodeLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            node_id: self.node_id.clone(),
            operation_type: OperationType::StorageOperation {
                operation: operation.to_string(),
                key: key.to_string(),
                size_bytes,
            },
            duration_ms,
            success,
            details: HashMap::new(),
            error,
        };

        debug!(
            operation = %operation,
            key = %key,
            size_bytes = ?size_bytes,
            duration_ms = ?duration_ms,
            success = %success,
            "Storage operation logged"
        );

        {
            let mut logs = self.logs.write().await;
            logs.push(entry);
        } // Block scope ends here, releasing the borrow

        // Update metrics
        let mut metrics = self.performance_metrics.write().await;
        metrics.storage_operations += 1;
        if let Some(size) = size_bytes {
            metrics.data_stored_bytes += size as u64;
        }
    }

    /// Get recent logs (last N entries)
    pub async fn get_recent_logs(&self, limit: usize) -> Vec<StorageNodeLogEntry> {
        let logs = self.logs.read().await;
        logs.iter().rev().take(limit).cloned().collect()
    }

    /// Get logs filtered by operation type
    pub async fn get_logs_by_type(
        &self,
        operation_type: &str,
        limit: usize,
    ) -> Vec<StorageNodeLogEntry> {
        let logs = self.logs.read().await;
        logs.iter()
            .rev()
            .filter(|log| match &log.operation_type {
                OperationType::ClientRequest { .. } => operation_type == "client_request",
                OperationType::NodeCommunication { .. } => operation_type == "node_communication",
                OperationType::MpcOperation { .. } => operation_type == "mpc_operation",
                OperationType::StorageOperation { .. } => operation_type == "storage_operation",
                OperationType::TopologyChange { .. } => operation_type == "topology_change",
            })
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> NodePerformanceMetrics {
        let mut metrics = self.performance_metrics.write().await;

        // Update uptime
        metrics.uptime_seconds = self.start_time.elapsed().unwrap_or_default().as_secs();
        metrics.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        metrics.clone()
    }

    /// Update request metrics
    async fn update_request_metrics(&self, success: bool, duration_ms: Option<u64>) {
        let mut metrics = self.performance_metrics.write().await;
        metrics.total_requests += 1;

        if success {
            metrics.successful_requests += 1;
        } else {
            metrics.failed_requests += 1;
        }

        // Update average response time
        if let Some(duration) = duration_ms {
            let total_responses = metrics.successful_requests + metrics.failed_requests;
            metrics.average_response_time_ms =
                (metrics.average_response_time_ms * (total_responses - 1) as f64 + duration as f64)
                    / total_responses as f64;
        }
    }

    /// Clean up old logs (keep only last N entries)
    pub async fn cleanup_old_logs(&self, keep_count: usize) {
        let mut logs = self.logs.write().await;
        let current_len = logs.len();
        if current_len > keep_count {
            logs.drain(0..current_len - keep_count);
        }
    }

    /// Export logs as JSON for external analysis
    pub async fn export_logs_json(&self, limit: Option<usize>) -> String {
        let logs = self.logs.read().await;
        let export_logs: Vec<_> = if let Some(limit) = limit {
            logs.iter().rev().take(limit).cloned().collect()
        } else {
            logs.clone()
        };

        serde_json::to_string_pretty(&export_logs).unwrap_or_else(|_| "[]".to_string())
    }
}

// Global monitor instance using OnceLock for thread safety
use std::sync::OnceLock;
static GLOBAL_MONITOR: OnceLock<Arc<StorageNodeMonitor>> = OnceLock::new();

/// Initialize global storage node monitor
pub fn init_monitor(node_id: String) {
    let monitor = Arc::new(StorageNodeMonitor::new(node_id));
    let _ = GLOBAL_MONITOR.set(monitor); // Ignore error if already set
}

/// Get reference to global monitor
pub fn get_monitor() -> Option<Arc<StorageNodeMonitor>> {
    GLOBAL_MONITOR.get().cloned()
}

/// Convenience macro for logging client requests
#[macro_export]
macro_rules! log_client_request {
    ($endpoint:expr, $method:expr, $client_ip:expr, $duration:expr, $success:expr, $details:expr, $error:expr) => {
        if let Some(monitor) = $crate::monitoring::get_monitor() {
            monitor
                .log_client_request($crate::monitoring::ClientRequestParams {
                    endpoint: $endpoint.to_string(),
                    method: $method.to_string(),
                    client_ip: $client_ip.to_string(),
                    duration_ms: $duration,
                    success: $success,
                    details: $details,
                    error: $error,
                })
                .await;
        }
    };
}

/// Convenience macro for logging node communication
#[macro_export]
macro_rules! log_node_communication {
    ($peer_id:expr, $operation:expr, $direction:expr, $duration:expr, $success:expr, $details:expr, $error:expr) => {
        if let Some(monitor) = $crate::monitoring::get_monitor() {
            monitor
                .log_node_communication($crate::monitoring::NodeCommunicationParams {
                    peer_node_id: $peer_id.to_string(),
                    operation: $operation.to_string(),
                    direction: $direction,
                    duration_ms: $duration,
                    success: $success,
                    details: $details,
                    error: $error,
                })
                .await;
        }
    };
}
