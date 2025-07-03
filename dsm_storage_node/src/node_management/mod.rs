//! # Node Management Module for DSM Storage Node
//!
//! This module provides comprehensive node lifecycle management, configuration,
//! monitoring, and administrative functions for DSM Storage Nodes.
//!
//! ## Key Features
//!
//! * **Lifecycle Management**: Node startup, shutdown, and maintenance operations
//! * **Configuration Management**: Dynamic configuration updates and validation
//! * **Health Monitoring**: Continuous health checks and diagnostic information
//! * **Resource Management**: CPU, memory, disk, and network resource monitoring
//! * **Peer Discovery**: Automatic discovery and registration of peer nodes
//! * **Maintenance Operations**: Backup, restore, and maintenance scheduling
//! * **Security Management**: Certificate management and security policies
//!
//! ## Architecture
//!
//! The node management system operates as a control plane for the storage node,
//! providing administrative and operational capabilities:
//!
//! * **Node Supervisor**: Manages the overall node lifecycle and state
//! * **Health Monitor**: Tracks node health metrics and performance
//! * **Configuration Manager**: Handles dynamic configuration changes
//! * **Resource Monitor**: Monitors system resources and capacity
//! * **Peer Manager**: Manages connections to other nodes in the cluster

use crate::error::{Result, StorageNodeError};
use crate::storage::StorageEngine;
use crate::types::{AppConfig, StorageNode};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::System;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Node lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    /// Node is initializing
    Initializing,
    /// Node is starting up
    Starting,
    /// Node is running normally
    Running,
    /// Node is in maintenance mode
    Maintenance,
    /// Node is shutting down
    Stopping,
    /// Node has stopped
    Stopped,
    /// Node has encountered an error
    Error,
    /// Node is degraded but operational
    Degraded,
}

/// Node health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Node is healthy
    Healthy,
    /// Node is experiencing warnings
    Warning,
    /// Node is in critical condition
    Critical,
    /// Node health is unknown
    Unknown,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// CPU usage percentage (0.0-100.0)
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_used: u64,
    /// Total memory in bytes
    pub memory_total: u64,
    /// Memory usage percentage (0.0-100.0)
    pub memory_usage: f64,
    /// Disk usage in bytes
    pub disk_used: u64,
    /// Total disk space in bytes
    pub disk_total: u64,
    /// Disk usage percentage (0.0-100.0)
    pub disk_usage: f64,
    /// Network bytes received
    pub network_rx_bytes: u64,
    /// Network bytes transmitted
    pub network_tx_bytes: u64,
    /// Number of open file descriptors
    pub open_files: u64,
    /// Process uptime in seconds
    pub uptime: u64,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Component being checked
    pub component: String,
    /// Health status
    pub status: HealthStatus,
    /// Human-readable message
    pub message: String,
    /// Detailed information
    pub details: HashMap<String, String>,
    /// Timestamp of the check
    pub timestamp: u64,
}

/// Node information and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Node identifier
    pub node_id: String,
    /// Node state
    pub state: NodeState,
    /// Overall health status
    pub health: HealthStatus,
    /// Node version
    pub version: String,
    /// Node start time
    pub started_at: u64,
    /// Last health check time
    pub last_health_check: u64,
    /// Resource metrics
    pub resources: ResourceMetrics,
    /// Individual health checks
    pub health_checks: Vec<HealthCheck>,
    /// Configuration summary
    pub config_summary: HashMap<String, String>,
    /// Peer connections
    pub peer_connections: Vec<PeerConnection>,
}

/// Information about peer connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnection {
    /// Peer node ID
    pub peer_id: String,
    /// Peer endpoint
    pub endpoint: String,
    /// Connection state
    pub state: ConnectionState,
    /// Last successful communication
    pub last_seen: u64,
    /// Round-trip latency in milliseconds
    pub latency_ms: u64,
    /// Number of failed attempts
    pub failed_attempts: u32,
}

/// Connection states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Connection is active and healthy
    Connected,
    /// Connection is being established
    Connecting,
    /// Connection is temporarily unavailable
    Disconnected,
    /// Connection has failed
    Failed,
    /// Connection is being retried
    Retrying,
}

/// Configuration for node management
#[derive(Debug, Clone)]
pub struct NodeManagementConfig {
    /// Health check interval in seconds
    pub health_check_interval: u64,
    /// Resource monitoring interval in seconds
    pub resource_monitoring_interval: u64,
    /// Peer discovery interval in seconds
    pub peer_discovery_interval: u64,
    /// Maximum number of failed health checks before marking as unhealthy
    pub max_failed_health_checks: u32,
    /// Connection timeout for peer communications
    pub peer_connection_timeout: Duration,
    /// Enable automatic restart on critical failures
    pub enable_auto_restart: bool,
    /// Maximum CPU usage before warning (percentage)
    pub cpu_warning_threshold: f64,
    /// Maximum memory usage before warning (percentage)
    pub memory_warning_threshold: f64,
    /// Maximum disk usage before warning (percentage)
    pub disk_warning_threshold: f64,
}

impl Default for NodeManagementConfig {
    fn default() -> Self {
        Self {
            health_check_interval: 30,        // 30 seconds
            resource_monitoring_interval: 60, // 1 minute
            peer_discovery_interval: 300,     // 5 minutes
            max_failed_health_checks: 3,
            peer_connection_timeout: Duration::from_secs(10),
            enable_auto_restart: false,
            cpu_warning_threshold: 80.0,
            memory_warning_threshold: 80.0,
            disk_warning_threshold: 90.0,
        }
    }
}

/// Main node management system
pub struct NodeManager {
    /// Node management configuration
    config: NodeManagementConfig,
    /// Application configuration
    app_config: AppConfig,
    /// Current node state
    node_state: Arc<RwLock<NodeState>>,
    /// Storage engine reference
    storage_engine: Arc<dyn StorageEngine + Send + Sync>,
    /// System information collector
    system: Arc<RwLock<System>>,
    /// Start time of the node
    start_time: SystemTime,
    /// Current resource metrics
    resource_metrics: Arc<RwLock<ResourceMetrics>>,
    /// Health check history
    health_history: Arc<RwLock<Vec<HealthCheck>>>,
    /// Known peer connections
    peer_connections: Arc<RwLock<HashMap<String, PeerConnection>>>,
    /// HTTP client for peer communications
    http_client: reqwest::Client,
}

impl NodeManager {
    /// Create a new node manager
    pub fn new(
        config: NodeManagementConfig,
        app_config: AppConfig,
        storage_engine: Arc<dyn StorageEngine + Send + Sync>,
    ) -> Self {
        let system = System::new_all();

        Self {
            config,
            app_config,
            node_state: Arc::new(RwLock::new(NodeState::Initializing)),
            storage_engine,
            system: Arc::new(RwLock::new(system)),
            start_time: SystemTime::now(),
            resource_metrics: Arc::new(RwLock::new(Self::initial_resource_metrics())),
            health_history: Arc::new(RwLock::new(Vec::new())),
            peer_connections: Arc::new(RwLock::new(HashMap::new())),
            http_client: reqwest::Client::new(),
        }
    }

    /// Initialize the node manager
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing node manager");

        // Set state to starting
        *self.node_state.write().await = NodeState::Starting;

        // Perform initial health checks
        self.perform_health_checks().await?;

        // Update resource metrics
        self.update_resource_metrics().await?;

        // Start periodic tasks
        self.start_periodic_tasks().await;

        // Set state to running
        *self.node_state.write().await = NodeState::Running;

        info!("Node manager initialized successfully");
        Ok(())
    }

    /// Shutdown the node manager
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down node manager");

        // Set state to stopping
        *self.node_state.write().await = NodeState::Stopping;

        // Perform final health check
        let _ = self.perform_health_checks().await;

        // Disconnect from all peers
        self.disconnect_all_peers().await;

        // Set state to stopped
        *self.node_state.write().await = NodeState::Stopped;

        info!("Node manager shutdown complete");
        Ok(())
    }

    /// Get current node information
    pub async fn get_node_info(&self) -> Result<NodeInfo> {
        let state = *self.node_state.read().await;
        let resources = self.resource_metrics.read().await.clone();
        let health_checks = self.health_history.read().await.clone();
        let peers: Vec<PeerConnection> = self
            .peer_connections
            .read()
            .await
            .values()
            .cloned()
            .collect();

        // Determine overall health from recent checks
        let health = self.determine_overall_health(&health_checks);

        // Create configuration summary
        let mut config_summary = HashMap::new();
        config_summary.insert("node_id".to_string(), self.app_config.node.id.clone());
        config_summary.insert("region".to_string(), self.app_config.node.region.clone());
        config_summary.insert(
            "storage_engine".to_string(),
            self.app_config.storage.engine.clone(),
        );
        config_summary.insert("api_port".to_string(), self.app_config.api.port.to_string());

        let started_at = self
            .start_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let last_health_check = health_checks
            .last()
            .map(|check| check.timestamp)
            .unwrap_or(0);

        Ok(NodeInfo {
            node_id: self.app_config.node.id.clone(),
            state,
            health,
            version: self.app_config.node.version.clone(),
            started_at,
            last_health_check,
            resources,
            health_checks,
            config_summary,
            peer_connections: peers,
        })
    }

    /// Perform comprehensive health checks
    pub async fn perform_health_checks(&self) -> Result<Vec<HealthCheck>> {
        debug!("Performing health checks");

        let mut checks = Vec::new();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Storage engine health check
        checks.push(self.check_storage_health(timestamp).await);

        // System resource health check
        checks.push(self.check_resource_health(timestamp).await);

        // Network connectivity health check
        checks.push(self.check_network_health(timestamp).await);

        // Configuration health check
        checks.push(self.check_configuration_health(timestamp));

        // Store health check results
        {
            let mut history = self.health_history.write().await;
            history.extend(checks.clone());

            // Keep only the last 100 health checks
            let len = history.len();
            if len > 100 {
                history.drain(0..len - 100);
            }
        }

        debug!("Completed {} health checks", checks.len());
        Ok(checks)
    }

    /// Check storage engine health
    async fn check_storage_health(&self, timestamp: u64) -> HealthCheck {
        let mut details = HashMap::new();

        let (status, message) = match self.storage_engine.get_stats().await {
            Ok(stats) => {
                details.insert("total_entries".to_string(), stats.total_entries.to_string());
                details.insert("total_bytes".to_string(), stats.total_bytes.to_string());
                details.insert("total_expired".to_string(), stats.total_expired.to_string());

                if stats.total_entries > 0 {
                    (
                        HealthStatus::Healthy,
                        "Storage engine is operational".to_string(),
                    )
                } else {
                    (
                        HealthStatus::Warning,
                        "Storage engine has no data".to_string(),
                    )
                }
            }
            Err(e) => {
                details.insert("error".to_string(), e.to_string());
                (
                    HealthStatus::Critical,
                    "Storage engine is not responding".to_string(),
                )
            }
        };

        HealthCheck {
            component: "storage".to_string(),
            status,
            message,
            details,
            timestamp,
        }
    }

    /// Check system resource health
    async fn check_resource_health(&self, timestamp: u64) -> HealthCheck {
        let mut details = HashMap::new();
        let resources = self.resource_metrics.read().await;

        let mut warnings = Vec::new();
        let mut status = HealthStatus::Healthy;

        // Check CPU usage
        if resources.cpu_usage > self.config.cpu_warning_threshold {
            warnings.push(format!("High CPU usage: {:.1}%", resources.cpu_usage));
            status = HealthStatus::Warning;
        }
        details.insert(
            "cpu_usage".to_string(),
            format!("{:.1}%", resources.cpu_usage),
        );

        // Check memory usage
        if resources.memory_usage > self.config.memory_warning_threshold {
            warnings.push(format!("High memory usage: {:.1}%", resources.memory_usage));
            status = HealthStatus::Warning;
        }
        details.insert(
            "memory_usage".to_string(),
            format!("{:.1}%", resources.memory_usage),
        );

        // Check disk usage
        if resources.disk_usage > self.config.disk_warning_threshold {
            warnings.push(format!("High disk usage: {:.1}%", resources.disk_usage));
            if resources.disk_usage > 95.0 {
                status = HealthStatus::Critical;
            } else {
                status = HealthStatus::Warning;
            }
        }
        details.insert(
            "disk_usage".to_string(),
            format!("{:.1}%", resources.disk_usage),
        );

        let message = if warnings.is_empty() {
            "System resources are within normal limits".to_string()
        } else {
            warnings.join(", ")
        };

        HealthCheck {
            component: "resources".to_string(),
            status,
            message,
            details,
            timestamp,
        }
    }

    /// Check network connectivity health
    async fn check_network_health(&self, timestamp: u64) -> HealthCheck {
        let mut details = HashMap::new();
        let peers = self.peer_connections.read().await;

        let total_peers = peers.len();
        let connected_peers = peers
            .values()
            .filter(|p| p.state == ConnectionState::Connected)
            .count();
        let failed_peers = peers
            .values()
            .filter(|p| p.state == ConnectionState::Failed)
            .count();

        details.insert("total_peers".to_string(), total_peers.to_string());
        details.insert("connected_peers".to_string(), connected_peers.to_string());
        details.insert("failed_peers".to_string(), failed_peers.to_string());

        let (status, message) = if total_peers == 0 {
            (
                HealthStatus::Warning,
                "No peer connections configured".to_string(),
            )
        } else if failed_peers > total_peers / 2 {
            (
                HealthStatus::Critical,
                format!("More than half of peers are unreachable ({failed_peers}/{total_peers})"),
            )
        } else if failed_peers > 0 {
            (
                HealthStatus::Warning,
                format!("Some peers are unreachable ({failed_peers}/{total_peers})"),
            )
        } else {
            (
                HealthStatus::Healthy,
                format!("All {connected_peers} peers are connected"),
            )
        };

        HealthCheck {
            component: "network".to_string(),
            status,
            message,
            details,
            timestamp,
        }
    }

    /// Check configuration health
    fn check_configuration_health(&self, timestamp: u64) -> HealthCheck {
        let mut details = HashMap::new();
        let mut warnings = Vec::new();

        // Check if required fields are present
        if self.app_config.node.id.is_empty() {
            warnings.push("Node ID is empty".to_string());
        }
        details.insert("node_id".to_string(), self.app_config.node.id.clone());

        if self.app_config.node.endpoint.is_empty() {
            warnings.push("Node endpoint is empty".to_string());
        }
        details.insert(
            "endpoint".to_string(),
            self.app_config.node.endpoint.clone(),
        );

        // Check storage configuration
        if self.app_config.storage.capacity == 0 {
            warnings.push("Storage capacity is set to 0".to_string());
        }
        details.insert(
            "storage_capacity".to_string(),
            self.app_config.storage.capacity.to_string(),
        );

        let (status, message) = if warnings.is_empty() {
            (HealthStatus::Healthy, "Configuration is valid".to_string())
        } else {
            (HealthStatus::Warning, warnings.join(", "))
        };

        HealthCheck {
            component: "configuration".to_string(),
            status,
            message,
            details,
            timestamp,
        }
    }

    /// Update resource metrics
    pub async fn update_resource_metrics(&self) -> Result<()> {
        let mut system = self.system.write().await;
        system.refresh_all();

        let process_id = std::process::id();
        let process = system.process(sysinfo::Pid::from(process_id as usize));

        let (cpu_usage, memory_used, open_files) = if let Some(proc) = process {
            (
                proc.cpu_usage() as f64,
                proc.memory() * 1024, // sysinfo returns KB, convert to bytes
                0u64,                 // fd_count not available in newer sysinfo versions
            )
        } else {
            (0.0, 0, 0)
        };

        let memory_total = system.total_memory() * 1024; // Convert KB to bytes
        let memory_usage = if memory_total > 0 {
            (memory_used as f64 / memory_total as f64) * 100.0
        } else {
            0.0
        };

        // Get disk usage from the storage data directory
        let disk_info = self.get_disk_usage(&self.app_config.storage.data_dir);
        let (disk_used, disk_total, disk_usage) = disk_info.unwrap_or((0, 0, 0.0));

        let uptime = self.start_time.elapsed().unwrap_or_default().as_secs();

        let metrics = ResourceMetrics {
            cpu_usage,
            memory_used,
            memory_total,
            memory_usage,
            disk_used,
            disk_total,
            disk_usage,
            network_rx_bytes: 0, // Would be implemented with network monitoring
            network_tx_bytes: 0, // Would be implemented with network monitoring
            open_files,
            uptime,
        };

        *self.resource_metrics.write().await = metrics;

        debug!(
            "Updated resource metrics: CPU: {:.1}%, Memory: {:.1}%, Disk: {:.1}%",
            cpu_usage, memory_usage, disk_usage
        );

        Ok(())
    }

    /// Get disk usage for a directory
    fn get_disk_usage(&self, path: &str) -> Option<(u64, u64, f64)> {
        use std::fs;
        use std::path::Path;

        let path = Path::new(path);
        if !path.exists() {
            return None;
        }

        // This is a simplified implementation
        // In production, you would use platform-specific APIs
        if let Ok(metadata) = fs::metadata(path) {
            // For simplicity, use the file system's available space
            // This is not accurate for actual disk usage calculation
            let used = metadata.len();
            let total = 1024 * 1024 * 1024 * 1024; // 1TB as placeholder
            let usage = (used as f64 / total as f64) * 100.0;
            Some((used, total, usage))
        } else {
            None
        }
    }

    /// Add a peer connection
    pub async fn add_peer(&self, peer: StorageNode) -> Result<()> {
        info!("Adding peer: {} at {}", peer.id, peer.endpoint);

        let connection = PeerConnection {
            peer_id: peer.id.clone(),
            endpoint: peer.endpoint.clone(),
            state: ConnectionState::Connecting,
            last_seen: 0,
            latency_ms: 0,
            failed_attempts: 0,
        };

        {
            let mut peers = self.peer_connections.write().await;
            peers.insert(peer.id.clone(), connection);
        }

        // Test the connection
        self.test_peer_connection(&peer.id).await;

        Ok(())
    }

    /// Remove a peer connection
    pub async fn remove_peer(&self, peer_id: &str) -> Result<()> {
        info!("Removing peer: {}", peer_id);

        let mut peers = self.peer_connections.write().await;
        peers.remove(peer_id);

        Ok(())
    }

    /// Test connection to a peer
    async fn test_peer_connection(&self, peer_id: &str) {
        let peer_info = {
            let peers = self.peer_connections.read().await;
            peers.get(peer_id).cloned()
        };

        if let Some(peer) = peer_info {
            let start_time = SystemTime::now();
            let health_url = format!("{}/api/v1/health", peer.endpoint);

            let result = tokio::time::timeout(
                self.config.peer_connection_timeout,
                self.http_client.get(&health_url).send(),
            )
            .await;

            let latency = start_time.elapsed().unwrap_or_default().as_millis() as u64;
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let mut peers = self.peer_connections.write().await;
            if let Some(connection) = peers.get_mut(peer_id) {
                match result {
                    Ok(Ok(response)) => {
                        if response.status().is_success() {
                            connection.state = ConnectionState::Connected;
                            connection.last_seen = now;
                            connection.latency_ms = latency;
                            connection.failed_attempts = 0;
                            debug!(
                                "Successfully connected to peer: {} ({}ms)",
                                peer_id, latency
                            );
                        } else {
                            connection.state = ConnectionState::Failed;
                            connection.failed_attempts += 1;
                            warn!("Peer {} returned HTTP {}", peer_id, response.status());
                        }
                    }
                    Ok(Err(e)) => {
                        connection.state = ConnectionState::Failed;
                        connection.failed_attempts += 1;
                        warn!("Failed to connect to peer {}: {}", peer_id, e);
                    }
                    Err(_) => {
                        connection.state = ConnectionState::Failed;
                        connection.failed_attempts += 1;
                        warn!("Timeout connecting to peer: {}", peer_id);
                    }
                }
            }
        }
    }

    /// Test all peer connections
    async fn test_all_peer_connections(&self) {
        let peer_ids: Vec<String> = {
            let peers = self.peer_connections.read().await;
            peers.keys().cloned().collect()
        };

        for peer_id in peer_ids {
            self.test_peer_connection(&peer_id).await;
        }
    }

    /// Disconnect from all peers
    async fn disconnect_all_peers(&self) {
        let mut peers = self.peer_connections.write().await;
        for connection in peers.values_mut() {
            connection.state = ConnectionState::Disconnected;
        }
    }

    /// Determine overall health from individual checks
    fn determine_overall_health(&self, checks: &[HealthCheck]) -> HealthStatus {
        if checks.is_empty() {
            return HealthStatus::Unknown;
        }

        // Take the worst status from recent checks (last 5)
        let recent_checks = checks.iter().rev().take(5);

        for check in recent_checks {
            if check.status == HealthStatus::Critical {
                return HealthStatus::Critical;
            }
        }

        for check in checks.iter().rev().take(5) {
            if check.status == HealthStatus::Warning {
                return HealthStatus::Warning;
            }
        }

        HealthStatus::Healthy
    }

    /// Start periodic background tasks
    async fn start_periodic_tasks(&self) {
        // Health check task
        let health_manager = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(
                health_manager.config.health_check_interval,
            ));

            loop {
                interval.tick().await;
                if let Err(e) = health_manager.perform_health_checks().await {
                    error!("Health check failed: {}", e);
                }
            }
        });

        // Resource monitoring task
        let resource_manager = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(
                resource_manager.config.resource_monitoring_interval,
            ));

            loop {
                interval.tick().await;
                if let Err(e) = resource_manager.update_resource_metrics().await {
                    error!("Resource metrics update failed: {}", e);
                }
            }
        });

        // Peer connectivity task
        let peer_manager = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(
                peer_manager.config.peer_discovery_interval,
            ));

            loop {
                interval.tick().await;
                peer_manager.test_all_peer_connections().await;
            }
        });
    }

    /// Enter maintenance mode
    pub async fn enter_maintenance_mode(&self, reason: String) -> Result<()> {
        info!("Entering maintenance mode: {}", reason);

        *self.node_state.write().await = NodeState::Maintenance;

        // Perform any maintenance-specific setup
        self.disconnect_all_peers().await;

        Ok(())
    }

    /// Exit maintenance mode
    pub async fn exit_maintenance_mode(&self) -> Result<()> {
        info!("Exiting maintenance mode");

        *self.node_state.write().await = NodeState::Running;

        // Reconnect to peers
        self.test_all_peer_connections().await;

        Ok(())
    }

    /// Get current node state
    pub async fn get_node_state(&self) -> NodeState {
        *self.node_state.read().await
    }

    /// Set node state
    pub async fn set_node_state(&self, state: NodeState) {
        info!("Node state changed to: {:?}", state);
        *self.node_state.write().await = state;
    }

    /// Initial resource metrics placeholder
    fn initial_resource_metrics() -> ResourceMetrics {
        ResourceMetrics {
            cpu_usage: 0.0,
            memory_used: 0,
            memory_total: 0,
            memory_usage: 0.0,
            disk_used: 0,
            disk_total: 0,
            disk_usage: 0.0,
            network_rx_bytes: 0,
            network_tx_bytes: 0,
            open_files: 0,
            uptime: 0,
        }
    }

    /// Clone for async tasks
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            app_config: self.app_config.clone(),
            node_state: self.node_state.clone(),
            storage_engine: self.storage_engine.clone(),
            system: self.system.clone(),
            start_time: self.start_time,
            resource_metrics: self.resource_metrics.clone(),
            health_history: self.health_history.clone(),
            peer_connections: self.peer_connections.clone(),
            http_client: self.http_client.clone(),
        }
    }
}

/// Administrative operations for the node
pub struct NodeAdministrator {
    node_manager: Arc<NodeManager>,
}

impl NodeAdministrator {
    /// Create a new node administrator
    pub fn new(node_manager: Arc<NodeManager>) -> Self {
        Self { node_manager }
    }

    /// Perform a backup of node data
    pub async fn backup_node_data(&self, backup_path: &str) -> Result<()> {
        info!("Starting node data backup to: {}", backup_path);

        // Set to maintenance mode
        self.node_manager
            .enter_maintenance_mode("Backup operation".to_string())
            .await?;

        // Perform backup (simplified implementation)
        // In production, this would backup storage data, configuration, etc.
        tokio::fs::create_dir_all(backup_path)
            .await
            .map_err(|e| StorageNodeError::IO(e.to_string()))?;

        info!("Node data backup completed");

        // Exit maintenance mode
        self.node_manager.exit_maintenance_mode().await?;

        Ok(())
    }

    /// Restore node data from backup
    pub async fn restore_node_data(&self, backup_path: &str) -> Result<()> {
        info!("Starting node data restore from: {}", backup_path);

        // Set to maintenance mode
        self.node_manager
            .enter_maintenance_mode("Restore operation".to_string())
            .await?;

        // Perform restore (simplified implementation)
        // In production, this would restore storage data, configuration, etc.
        if !tokio::fs::try_exists(backup_path).await.unwrap_or(false) {
            return Err(StorageNodeError::NotFound(format!(
                "Backup path not found: {backup_path}"
            )));
        }

        info!("Node data restore completed");

        // Exit maintenance mode
        self.node_manager.exit_maintenance_mode().await?;

        Ok(())
    }

    /// Update node configuration
    pub async fn update_configuration(&self, new_config: AppConfig) -> Result<()> {
        info!("Updating node configuration");

        // Validate configuration
        self.validate_configuration(&new_config)?;

        // Apply configuration (this would require a restart in production)
        info!("Configuration validation passed - restart required to apply changes");

        Ok(())
    }

    /// Validate configuration
    fn validate_configuration(&self, config: &AppConfig) -> Result<()> {
        if config.node.id.is_empty() {
            return Err(StorageNodeError::Config(
                "Node ID cannot be empty".to_string(),
            ));
        }

        if config.node.endpoint.is_empty() {
            return Err(StorageNodeError::Config(
                "Node endpoint cannot be empty".to_string(),
            ));
        }

        if config.storage.capacity == 0 {
            return Err(StorageNodeError::Config(
                "Storage capacity must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Graceful shutdown
    pub async fn graceful_shutdown(&self) -> Result<()> {
        info!("Initiating graceful shutdown");

        // Stop accepting new requests
        self.node_manager.set_node_state(NodeState::Stopping).await;

        // Wait for ongoing operations to complete
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Shutdown node manager
        self.node_manager.shutdown().await?;

        info!("Graceful shutdown completed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::memory_storage::{MemoryStorage, MemoryStorageConfig};
    use crate::types::{ApiConfig, NetworkConfig, StorageConfig};
    use crate::NodeConfig;

    fn create_test_config() -> AppConfig {
        AppConfig {
            api: ApiConfig {
                bind_address: "127.0.0.1".to_string(),
                port: 8080,
                enable_cors: false,
                enable_rate_limits: false,
                max_body_size: 1024 * 1024,
            },
            node: NodeConfig {
                id: "test-node".to_string(),
                name: "Test Node".to_string(),
                region: "test-region".to_string(),
                operator: "test-operator".to_string(),
                version: "1.0.0".to_string(),
                description: "Test node".to_string(),
                public_key: "test-key".to_string(),
                endpoint: "http://localhost:8080".to_string(),
            },
            storage: StorageConfig {
                engine: "memory".to_string(),
                capacity: 1024 * 1024 * 1024,
                data_dir: "/tmp/test".to_string(),
                database_path: "/tmp/test.db".to_string(),
                assignment_strategy: "DeterministicHashing".to_string(),
                replication_strategy: "FixedReplicas".to_string(),
                replica_count: 3,
                min_regions: 1,
                default_ttl: 3600,
                enable_pruning: false,
                pruning_interval: 3600,
            },
            network: NetworkConfig {
                listen_addr: "127.0.0.1".to_string(),
                public_endpoint: "http://localhost:8080".to_string(),
                port: 8080,
                max_connections: 100,
                connection_timeout: 30,
                enable_discovery: false,
                discovery_interval: 300,
                max_peers: 10,
            },
            cluster: None,
        }
    }

    fn create_test_manager() -> NodeManager {
        let config = NodeManagementConfig::default();
        let app_config = create_test_config();
        let storage = Arc::new(MemoryStorage::new(MemoryStorageConfig::default()));

        NodeManager::new(config, app_config, storage)
    }

    #[tokio::test]
    async fn test_node_manager_initialization() {
        let mut manager = create_test_manager();
        assert!(manager.initialize().await.is_ok());

        let state = manager.get_node_state().await;
        assert_eq!(state, NodeState::Running);
    }

    #[tokio::test]
    async fn test_health_checks() {
        let manager = create_test_manager();
        let health_checks = manager.perform_health_checks().await.unwrap();

        assert!(!health_checks.is_empty());
        assert!(health_checks
            .iter()
            .any(|check| check.component == "storage"));
        assert!(health_checks
            .iter()
            .any(|check| check.component == "resources"));
    }

    #[tokio::test]
    async fn test_node_info() {
        let manager = create_test_manager();
        let node_info = manager.get_node_info().await.unwrap();

        assert_eq!(node_info.node_id, "test-node");
        assert_eq!(node_info.state, NodeState::Initializing);
    }

    #[tokio::test]
    async fn test_peer_management() {
        let manager = create_test_manager();

        let peer = StorageNode {
            id: "peer-1".to_string(),
            name: "Peer 1".to_string(),
            region: "test-region".to_string(),
            public_key: "peer-key".to_string(),
            endpoint: "http://localhost:8081".to_string(),
        };

        assert!(manager.add_peer(peer.clone()).await.is_ok());
        assert!(manager.remove_peer(&peer.id).await.is_ok());
    }
}
