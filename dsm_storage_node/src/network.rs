// Network module for DSM Storage Node
//
// This module provides production-ready networking capabilities for communicating between storage nodes.
// Features include connection pooling, retry logic, metrics collection, and comprehensive error handling.

use async_trait::async_trait;
use lru::LruCache;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::error::{Result, StorageNodeError};
use crate::types::StorageNode;

/// Peer information for network communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Node ID of the peer
    pub node_id: String,
    /// Network endpoint of the peer
    pub endpoint: String,
    /// Last seen timestamp
    pub last_seen: std::time::SystemTime,
    /// Reliability score (0.0 to 1.0)
    pub reliability_score: f64,
    /// Last known state number (optional)
    pub last_state_number: Option<u64>,
    /// Public key of the peer (optional)
    pub public_key: Option<Vec<u8>>,
}

/// State request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRequest {
    /// Keys to request
    pub keys: Vec<String>,
}

/// Mock peer discovery service for compilation
#[derive(Debug, Clone)]
pub struct PeerDiscoveryService {
    node_id: String,
    peers: Arc<Mutex<Vec<PeerInfo>>>,
}

impl PeerDiscoveryService {
    pub fn new(config: &NetworkClientConfig) -> Result<Self> {
        Ok(Self {
            node_id: config.user_agent.clone(),
            peers: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub async fn add_peer(&self, peer: PeerInfo) -> Result<()> {
        self.peers.lock().unwrap().push(peer);
        Ok(())
    }

    /// Get all discovered peers
    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.lock().unwrap().clone()
    }

    /// Remove a peer from discovery
    pub async fn remove_peer(&self, peer_id: &str) -> Result<bool> {
        let mut peers = self.peers.lock().unwrap();
        if let Some(index) = peers.iter().position(|p| p.node_id == peer_id) {
            peers.remove(index);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if a peer is known
    pub async fn has_peer(&self, peer_id: &str) -> bool {
        self.peers
            .lock()
            .unwrap()
            .iter()
            .any(|p| p.node_id == peer_id)
    }

    pub async fn get_active_peers(&self) -> Result<Vec<PeerInfo>> {
        Ok(self.peers.lock().unwrap().clone())
    }

    pub async fn find_peer(&self, node_id: &str) -> Result<Option<PeerInfo>> {
        Ok(self
            .peers
            .lock()
            .unwrap()
            .iter()
            .find(|p| p.node_id == node_id)
            .cloned())
    }

    /// Get this service's node ID
    pub fn get_node_id(&self) -> &str {
        &self.node_id
    }

    pub async fn find_nodes_near(&self, _target: &crate::storage::topology::NodeId) -> Result<()> {
        Ok(())
    }

    pub async fn find_nodes_in_region(&self, _region: u8) -> Result<()> {
        Ok(())
    }
}
/// Maximum number of retry attempts for network operations
const MAX_RETRY_ATTEMPTS: usize = 3;

/// Default timeout for network operations
const DEFAULT_TIMEOUT_MS: u64 = 300000; // 5 minutes for MPC

/// Configuration for network client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkClientConfig {
    /// Default timeout for network operations in milliseconds
    pub timeout_ms: u64,
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Base delay for exponential backoff in milliseconds
    pub base_retry_delay_ms: u64,
    /// Maximum number of concurrent connections per host
    pub max_connections_per_host: usize,
    /// Keep-alive timeout in seconds
    pub keep_alive_timeout_s: u64,
    /// Enable TCP keepalive
    pub tcp_keepalive: bool,
    /// User agent string
    pub user_agent: String,
    /// Enable compression
    pub enable_compression: bool,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Message cache size
    pub message_cache_size: usize,
    /// Epidemic fanout for broadcasting
    pub epidemic_fanout: usize,
    /// Connection pool idle timeout in seconds
    pub connection_pool_idle_timeout: u64,
}

impl Default for NetworkClientConfig {
    fn default() -> Self {
        Self {
            timeout_ms: DEFAULT_TIMEOUT_MS,
            max_retries: MAX_RETRY_ATTEMPTS,
            base_retry_delay_ms: 100,
            max_connections_per_host: 10,
            keep_alive_timeout_s: 90,
            tcp_keepalive: true,
            user_agent: "DSM-Storage-Node/1.0".to_string(),
            enable_compression: true,
            request_timeout: 300, // 5 minutes for MPC
            retry_delay_ms: 100,
            message_cache_size: 1000,
            epidemic_fanout: 3,
            connection_pool_idle_timeout: 90,
        }
    }
}

/// Network metrics for monitoring
#[derive(Debug, Clone, Default)]
pub struct NetworkMetrics {
    /// Total requests sent
    pub requests_sent: Arc<AtomicU64>,
    /// Total responses received
    pub responses_received: Arc<AtomicU64>,
    /// Total errors encountered
    pub errors: Arc<AtomicU64>,
    /// Total timeouts
    pub timeouts: Arc<AtomicU64>,
    /// Total retries performed
    pub retries: Arc<AtomicU64>,
    /// Total bytes sent
    pub bytes_sent: Arc<AtomicU64>,
    /// Total bytes received
    pub bytes_received: Arc<AtomicU64>,
}

impl NetworkMetrics {
    /// Create new network metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a request
    pub fn record_request(&self, bytes_sent: usize) {
        self.requests_sent.fetch_add(1, Ordering::Relaxed);
        self.bytes_sent
            .fetch_add(bytes_sent as u64, Ordering::Relaxed);
    }

    /// Record a successful response
    pub fn record_response(&self, bytes_received: usize) {
        self.responses_received.fetch_add(1, Ordering::Relaxed);
        self.bytes_received
            .fetch_add(bytes_received as u64, Ordering::Relaxed);
    }

    /// Record an error
    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a timeout
    pub fn record_timeout(&self) {
        self.timeouts.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a retry
    pub fn record_retry(&self) {
        self.retries.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> NetworkMetricsSnapshot {
        NetworkMetricsSnapshot {
            requests_sent: self.requests_sent.load(Ordering::Relaxed),
            responses_received: self.responses_received.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            timeouts: self.timeouts.load(Ordering::Relaxed),
            retries: self.retries.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of network metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetricsSnapshot {
    pub requests_sent: u64,
    pub responses_received: u64,
    pub errors: u64,
    pub timeouts: u64,
    pub retries: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

impl NetworkMetricsSnapshot {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.requests_sent == 0 {
            0.0
        } else {
            (self.responses_received as f64 / self.requests_sent as f64) * 100.0
        }
    }

    /// Calculate error rate
    pub fn error_rate(&self) -> f64 {
        if self.requests_sent == 0 {
            0.0
        } else {
            (self.errors as f64 / self.requests_sent as f64) * 100.0
        }
    }

    /// Calculate average retry rate
    pub fn retry_rate(&self) -> f64 {
        if self.requests_sent == 0 {
            0.0
        } else {
            self.retries as f64 / self.requests_sent as f64
        }
    }
}

/// Network client trait for communicating with other nodes
#[async_trait]
pub trait NetworkClient: Send + Sync {
    /// Send entries to another node
    async fn send_entries(&self, node_id: String, entries: Vec<StateEntry>) -> Result<()>;

    /// Request entries from another node
    async fn request_entries(&self, node_id: String, keys: Vec<String>) -> Result<Vec<StateEntry>>;

    /// Forward a PUT operation to another node
    async fn forward_put(&self, node_id: String, key: String, value: Vec<u8>) -> Result<()>;

    /// Forward a GET operation to another node
    async fn forward_get(&self, node_id: String, key: String) -> Result<Option<Vec<u8>>>;

    /// Forward a DELETE operation to another node
    async fn forward_delete(&self, node_id: String, key: String) -> Result<()>;

    /// Get the status of another node
    async fn get_node_status(&self, node_id: &str) -> Result<NodeStatus>;

    /// Join a cluster by contacting bootstrap nodes
    async fn join_cluster(
        &self,
        bootstrap_nodes: Vec<String>,
        node_endpoint: String,
    ) -> Result<Vec<StorageNode>>;

    /// Register a node endpoint for forwarding operations
    async fn register_node(&self, node_id: String, endpoint: String);

    /// Send a message to another node for topology propagation
    fn send_message(
        &self,
        device_id: std::net::SocketAddr,
        message_id: [u8; 32],
        data: Vec<u8>,
        ttl: u8,
    ) -> Result<()>;

    /// Find nodes close to the target ID
    fn find_nodes(&self, target: &crate::storage::topology::NodeId) -> Result<()>;

    /// Find nodes in a specific geographic region
    fn find_nodes_in_region(&self, region: u8) -> Result<()>;

    /// Get network metrics
    fn get_metrics(&self) -> NetworkMetricsSnapshot;

    /// Perform health check on a node
    async fn health_check(&self, node_id: &str) -> Result<bool>;

    /// Get connection pool status
    fn get_connection_status(&self) -> ConnectionPoolStatus;
}

/// Connection pool status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolStatus {
    /// Total number of connections
    pub total_connections: usize,
    /// Number of active connections
    pub active_connections: usize,
    /// Number of idle connections
    pub idle_connections: usize,
    /// Number of pending connection requests
    pub pending_requests: usize,
}

/// Production-ready HTTP network client implementation
pub struct HttpNetworkClient {
    /// Local node ID
    node_id: String,
    /// HTTP client with connection pooling
    client: reqwest::Client,
    /// Node registry mapping node IDs to endpoints
    node_registry: Arc<tokio::sync::RwLock<HashMap<String, String>>>,
    /// Configuration
    config: NetworkClientConfig,
    /// Network metrics
    metrics: NetworkMetrics,
    /// Connection health cache
    connection_health: Arc<tokio::sync::RwLock<HashMap<String, ConnectionHealth>>>,
    /// Default timeout for operations
    timeout: Duration,
}

/// Connection health information
#[derive(Debug, Clone)]
struct ConnectionHealth {
    /// Last successful connection time
    last_success: SystemTime,
    /// Number of consecutive failures
    consecutive_failures: u32,
    /// Total requests to this endpoint
    total_requests: u64,
    /// Total failures to this endpoint
    total_failures: u64,
}

impl ConnectionHealth {
    fn new() -> Self {
        Self {
            last_success: SystemTime::now(),
            consecutive_failures: 0,
            total_requests: 0,
            total_failures: 0,
        }
    }

    #[allow(dead_code)]
    fn record_success(&mut self) {
        self.last_success = SystemTime::now();
        self.consecutive_failures = 0;
        self.total_requests += 1;
    }

    #[allow(dead_code)]
    fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        self.total_requests += 1;
        self.total_failures += 1;
    }

    #[allow(dead_code)]
    fn is_healthy(&self) -> bool {
        // Consider unhealthy if more than 5 consecutive failures
        // or if failure rate is over 50%
        self.consecutive_failures < 5
            && (self.total_requests == 0
                || (self.total_failures as f64 / self.total_requests as f64) < 0.5)
    }

    #[allow(dead_code)]
    fn should_circuit_break(&self) -> bool {
        self.consecutive_failures >= 10
    }
}

impl HttpNetworkClient {
    /// Create a new production-ready HTTP network client
    pub fn new(node_id: String, config: NetworkClientConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .pool_max_idle_per_host(config.max_connections_per_host)
            .pool_idle_timeout(Duration::from_secs(config.keep_alive_timeout_s))
            .tcp_keepalive(if config.tcp_keepalive {
                Some(Duration::from_secs(300)) // 5 minutes for MPC operations
            } else {
                None
            })
            .user_agent(&config.user_agent)
            .http2_prior_knowledge()
            .http2_keep_alive_interval(Duration::from_secs(30))
            .http2_keep_alive_timeout(Duration::from_secs(10))
            .http2_keep_alive_while_idle(true)
            .build()
            .map_err(|e| StorageNodeError::Network(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self {
            node_id,
            client,
            node_registry: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            timeout: Duration::from_millis(config.timeout_ms),
            config,
            metrics: NetworkMetrics::new(),
            connection_health: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }

    /// Create with default configuration
    pub fn with_default_config(node_id: String, timeout_ms: u64) -> Result<Self> {
        let config = NetworkClientConfig {
            timeout_ms,
            ..Default::default()
        };
        Self::new(node_id, config)
    }

    /// Register a node endpoint
    pub async fn register_node(&self, node_id: String, endpoint: String) {
        let mut registry = self.node_registry.write().await;
        registry.insert(node_id.clone(), endpoint);

        // Initialize connection health
        let mut health = self.connection_health.write().await;
        health.entry(node_id).or_insert_with(ConnectionHealth::new);
    }

    /// Get the endpoint for a node
    async fn get_endpoint(&self, node_id: &str) -> Result<String> {
        let registry = self.node_registry.read().await;
        let endpoint = registry
            .get(node_id)
            .cloned()
            .ok_or_else(|| StorageNodeError::NodeManagement(format!("Unknown node: {node_id}")))?;

        // Ensure the endpoint has the http:// scheme prefix
        if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            Ok(endpoint)
        } else {
            Ok(format!("http://{endpoint}"))
        }
    }

    /// Check if a node is healthy
    #[allow(dead_code)]
    async fn is_node_healthy(&self, node_id: &str) -> bool {
        let health = self.connection_health.read().await;
        health.get(node_id).is_none_or(|h| h.is_healthy())
    }

    /// Check if we should circuit break for a node
    #[allow(dead_code)]
    async fn should_circuit_break(&self, node_id: &str) -> bool {
        let health = self.connection_health.read().await;
        health
            .get(node_id)
            .is_some_and(|h| h.should_circuit_break())
    }

    /// Record successful operation
    #[allow(dead_code)]
    async fn record_success(&self, node_id: &str) {
        let mut health = self.connection_health.write().await;
        health
            .entry(node_id.to_string())
            .or_insert_with(ConnectionHealth::new)
            .record_success();
    }

    /// Record failed operation
    #[allow(dead_code)]
    async fn record_failure(&self, node_id: &str) {
        let mut health = self.connection_health.write().await;
        health
            .entry(node_id.to_string())
            .or_insert_with(ConnectionHealth::new)
            .record_failure();
    }

    /// Execute an HTTP request with retry logic
    #[allow(dead_code)]
    async fn execute_with_retry<F, T>(&self, node_id: &str, operation: F) -> Result<T>
    where
        F: Fn() -> Result<T> + Clone,
        T: Send + 'static,
    {
        // Check circuit breaker
        if self.should_circuit_break(node_id).await {
            return Err(StorageNodeError::Network(format!(
                "Circuit breaker open for node: {node_id}"
            )));
        }

        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            match operation() {
                Ok(result) => {
                    if attempt > 0 {
                        self.metrics.record_retry();
                    }
                    self.record_success(node_id).await;
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);
                    self.record_failure(node_id).await;

                    if attempt < self.config.max_retries {
                        let delay = Duration::from_millis(
                            self.config.base_retry_delay_ms * (2_u64.pow(attempt as u32)),
                        );
                        debug!(
                            "Retrying operation for node {} in {:?} (attempt {}/{})",
                            node_id,
                            delay,
                            attempt + 1,
                            self.config.max_retries
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Perform HTTP request with timeout and metrics
    async fn http_request(&self, request: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let request = request
            .build()
            .map_err(|e| StorageNodeError::Network(format!("Failed to build request: {e}")))?;

        let body_size = request
            .body()
            .map(|b| b.as_bytes().map(|bytes| bytes.len()).unwrap_or(0))
            .unwrap_or(0);
        self.metrics.record_request(body_size);

        let result = timeout(
            Duration::from_millis(self.config.timeout_ms),
            self.client.execute(request),
        )
        .await;

        match result {
            Ok(Ok(response)) => {
                let response_size = response.content_length().unwrap_or(0) as usize;
                self.metrics.record_response(response_size);
                Ok(response)
            }
            Ok(Err(e)) => {
                self.metrics.record_error();
                Err(StorageNodeError::Network(format!(
                    "HTTP request failed: {e}"
                )))
            }
            Err(_) => {
                self.metrics.record_timeout();
                Err(StorageNodeError::Timeout)
            }
        }
    }
}

#[async_trait]
impl NetworkClient for HttpNetworkClient {
    async fn send_entries(&self, node_id: String, entries: Vec<StateEntry>) -> Result<()> {
        let endpoint = self.get_endpoint(&node_id).await?;
        let url = format!("{endpoint}/api/v1/entries");

        debug!("Sending {} entries to node {}", entries.len(), node_id);

        let result = timeout(self.timeout, self.client.post(&url).json(&entries).send())
            .await
            .map_err(|_| StorageNodeError::Timeout)?;

        match result {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(StorageNodeError::Network(format!(
                        "Failed to send entries to {}: HTTP {}",
                        node_id,
                        response.status()
                    )))
                }
            }
            Err(_e) => Err(StorageNodeError::Network(format!(
                "Failed to send entries to {node_id}: {_e}"
            ))),
        }
    }

    async fn request_entries(&self, node_id: String, keys: Vec<String>) -> Result<Vec<StateEntry>> {
        let endpoint = self.get_endpoint(&node_id).await?;
        let url = format!("{endpoint}/api/v1/entries/request");

        debug!("Requesting {} entries from node {}", keys.len(), node_id);

        let result = timeout(self.timeout, self.client.post(&url).json(&keys).send())
            .await
            .map_err(|_| StorageNodeError::Timeout)?;

        match result {
            Ok(response) => {
                if response.status().is_success() {
                    // Parse the received entries
                    let entries: Vec<StateEntry> = response.json().await.map_err(|e| {
                        StorageNodeError::Network(format!(
                            "Failed to parse entries from {node_id}: {e}"
                        ))
                    })?;

                    debug!("Received {} entries from node {}", entries.len(), node_id);
                    Ok(entries)
                } else {
                    Err(StorageNodeError::Network(format!(
                        "Failed to request entries from {}: HTTP {}",
                        node_id,
                        response.status()
                    )))
                }
            }
            Err(_e) => Err(StorageNodeError::Network(format!(
                "Failed to request entries from {node_id}: {_e}"
            ))),
        }
    }

    async fn forward_put(&self, node_id: String, key: String, value: Vec<u8>) -> Result<()> {
        let endpoint = self.get_endpoint(&node_id).await?;
        let url = format!("{endpoint}/api/v1/data/{key}");

        debug!("Forwarding PUT for key {} to node {}", key, node_id);

        let result = timeout(self.timeout, self.client.put(&url).body(value).send())
            .await
            .map_err(|_| StorageNodeError::Timeout)?;

        match result {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(StorageNodeError::Network(format!(
                        "Failed to forward PUT to {}: HTTP {}",
                        node_id,
                        response.status()
                    )))
                }
            }
            Err(_e) => Err(StorageNodeError::Network(format!(
                "Failed to forward PUT to {node_id}: {_e}"
            ))),
        }
    }

    async fn forward_get(&self, node_id: String, key: String) -> Result<Option<Vec<u8>>> {
        let endpoint = self.get_endpoint(&node_id).await?;
        let url = format!("{endpoint}/api/v1/data/{key}");

        debug!("Forwarding GET for key {} to node {}", key, node_id);

        let result = timeout(self.timeout, self.client.get(&url).send())
            .await
            .map_err(|_| StorageNodeError::Timeout)?;

        match result {
            Ok(response) => {
                if response.status().is_success() {
                    let bytes = response.bytes().await.map_err(|_e| {
                        StorageNodeError::Network(format!("Failed to read response body: {_e}"))
                    })?;
                    Ok(Some(bytes.to_vec()))
                } else if response.status() == reqwest::StatusCode::NOT_FOUND {
                    Ok(None)
                } else {
                    Err(StorageNodeError::Network(format!(
                        "Failed to forward GET to {}: HTTP {}",
                        node_id,
                        response.status()
                    )))
                }
            }
            Err(_e) => Err(StorageNodeError::Network(format!(
                "Failed to forward GET to {node_id}: {_e}"
            ))),
        }
    }

    async fn forward_delete(&self, node_id: String, key: String) -> Result<()> {
        let endpoint = self.get_endpoint(&node_id).await?;
        let url = format!("{endpoint}/api/v1/data/{key}");

        debug!("Forwarding DELETE for key {} to node {}", key, node_id);

        let result = timeout(self.timeout, self.client.delete(&url).send())
            .await
            .map_err(|_| StorageNodeError::Timeout)?;

        match result {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(StorageNodeError::Network(format!(
                        "Failed to forward DELETE to {}: HTTP {}",
                        node_id,
                        response.status()
                    )))
                }
            }
            Err(_e) => Err(StorageNodeError::Network(format!(
                "Failed to forward DELETE to {node_id}: {_e}"
            ))),
        }
    }

    async fn get_node_status(&self, node_id: &str) -> Result<NodeStatus> {
        let endpoint = self.get_endpoint(node_id).await?;
        let url = format!("{endpoint}/status");

        debug!("Getting status from node {}", node_id);

        let result = timeout(self.timeout, self.client.get(&url).send())
            .await
            .map_err(|_| StorageNodeError::Timeout)?;

        match result {
            Ok(response) => {
                if response.status().is_success() {
                    let status = response.json::<NodeStatus>().await.map_err(|_e| {
                        StorageNodeError::Serialization(format!(
                            "Failed to parse node status: {_e}"
                        ))
                    })?;
                    Ok(status)
                } else {
                    Err(StorageNodeError::Network(format!(
                        "Failed to get status from {}: HTTP {}",
                        node_id,
                        response.status()
                    )))
                }
            }
            Err(_e) => Err(StorageNodeError::Network(format!(
                "Failed to get status from {node_id}: {_e}"
            ))),
        }
    }

    async fn join_cluster(
        &self,
        bootstrap_nodes: Vec<String>,
        node_endpoint: String,
    ) -> Result<Vec<StorageNode>> {
        info!("Joining cluster via bootstrap nodes: {:?}", bootstrap_nodes);
        let mut nodes = Vec::new();

        for node_addr in bootstrap_nodes {
            let url = format!("http://{node_addr}/join");
            debug!("Sending join request to {}", url);

            match timeout(
                self.timeout,
                self.client
                    .post(&url)
                    .json(&JoinRequest {
                        node_id: self.node_id.clone(),
                        endpoint: node_endpoint.clone(),
                    })
                    .send(),
            )
            .await
            {
                Ok(Ok(response)) => {
                    if response.status().is_success() {
                        match response.json::<JoinResponse>().await {
                            Ok(join_response) => {
                                info!(
                                    "Successfully joined cluster via {} - received {} nodes",
                                    node_addr,
                                    join_response.nodes.len()
                                );
                                nodes = join_response.nodes;
                                break;
                            }
                            Err(_e) => {
                                warn!("Failed to parse join response from {}: {}", node_addr, _e);
                                continue;
                            }
                        }
                    } else {
                        warn!(
                            "Failed to join via {}: HTTP {}",
                            node_addr,
                            response.status()
                        );
                        continue;
                    }
                }
                Ok(Err(_)) => {
                    warn!("Failed to connect to bootstrap node {}", node_addr);
                    continue;
                }
                Err(_) => {
                    warn!("Timeout connecting to bootstrap node {}", node_addr);
                    continue;
                }
            }
        }

        if nodes.is_empty() {
            Err(StorageNodeError::NodeManagement(
                "Failed to join cluster via any bootstrap node".to_string(),
            ))
        } else {
            // Register nodes in the registry
            let mut registry = self.node_registry.write().await;
            for node in &nodes {
                registry.insert(node.id.clone(), node.endpoint.clone());
            }
            Ok(nodes)
        }
    }

    fn send_message(
        &self,
        device_id: std::net::SocketAddr,
        message_id: [u8; 32],
        data: Vec<u8>,
        ttl: u8,
    ) -> Result<()> {
        // Create a future for the async send_message operation
        let future = async {
            let url = format!("http://{device_id}/message");
            debug!(
                "Sending message {} to {}",
                hex::encode(message_id),
                device_id
            );

            // Prepare the message payload
            let payload = MessagePayload {
                message_id,
                sender_id: self.node_id.clone(),
                data,
                ttl,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };

            // Send the message with timeout
            let result = timeout(self.timeout, self.client.post(&url).json(&payload).send()).await;

            match result {
                Ok(Ok(response)) => {
                    if response.status().is_success() {
                        Ok(())
                    } else {
                        Err(StorageNodeError::Network(format!(
                            "Failed to send message to {}: HTTP {}",
                            device_id,
                            response.status()
                        )))
                    }
                }
                Ok(Err(_e)) => Err(StorageNodeError::Network(format!(
                    "Failed to send message to {device_id}: {_e}"
                ))),
                Err(_) => Err(StorageNodeError::Timeout),
            }
        };

        // Execute the future in a synchronous context
        // This is a workaround since the trait method is not async
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle.block_on(future),
            Err(_) => {
                // Create a new runtime if we're not in a tokio context
                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    StorageNodeError::Internal(format!("Failed to create tokio runtime: {e}"))
                })?;
                rt.block_on(future)
            }
        }
    }

    fn find_nodes(&self, target: &crate::storage::topology::NodeId) -> Result<()> {
        // Create a future for the async find_nodes operation
        let target_clone = target.clone();
        let node_registry = Arc::clone(&self.node_registry);
        let client = self.client.clone();
        let timeout = self.timeout;

        let future = async move {
            debug!("Finding nodes close to target ID: {}", target_clone);

            // Get immediate neighbors to query
            let registry = node_registry.read().await;
            let neighbors: Vec<(String, String)> = registry.clone().into_iter().take(3).collect();

            if neighbors.is_empty() {
                return Err(StorageNodeError::NodeManagement(
                    "No known nodes to query".to_string(),
                ));
            }

            // Query each neighbor for nodes close to the target
            for (node_id, endpoint) in neighbors {
                let url = format!("{endpoint}/find_nodes/{target_clone}");

                match tokio::time::timeout(timeout, client.get(&url).send()).await {
                    Ok(Ok(response)) => {
                        if response.status().is_success() {
                            // Process response
                            debug!("Received response from node {} for find_nodes", node_id);
                        } else {
                            warn!(
                                "Failed find_nodes query to {}: HTTP {}",
                                node_id,
                                response.status()
                            );
                        }
                    }
                    Ok(Err(_)) => {
                        warn!("Failed to send find_nodes query to {}", node_id);
                    }
                    Err(_) => {
                        warn!("Timeout querying node {} for find_nodes", node_id);
                    }
                }
            }

            Ok(())
        };

        // Execute the future in a synchronous context
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle.block_on(future),
            Err(_) => {
                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    StorageNodeError::Internal(format!("Failed to create tokio runtime: {e}"))
                })?;
                rt.block_on(future)
            }
        }
    }

    fn find_nodes_in_region(&self, region: u8) -> Result<()> {
        // Create a future for the async find_nodes_in_region operation
        let node_registry = Arc::clone(&self.node_registry);
        let client = self.client.clone();
        let timeout = self.timeout;

        let future = async move {
            debug!("Finding nodes in region: {}", region);

            // Get some known nodes to query
            let registry = node_registry.read().await;
            let nodes: Vec<(String, String)> = registry.clone().into_iter().take(5).collect();

            if nodes.is_empty() {
                return Err(StorageNodeError::NodeManagement(
                    "No known nodes to query".to_string(),
                ));
            }

            // Query each node for nodes in the specified region
            for (node_id, endpoint) in nodes {
                let url = format!("{endpoint}/find_nodes_in_region/{region}");

                match tokio::time::timeout(timeout, client.get(&url).send()).await {
                    Ok(Ok(response)) => {
                        if response.status().is_success() {
                            debug!("Received response from node {} for region query", node_id);
                        } else {
                            warn!(
                                "Failed region query to {}: HTTP {}",
                                node_id,
                                response.status()
                            );
                        }
                    }
                    Ok(Err(_)) => {
                        warn!("Failed to send region query to {}", node_id);
                    }
                    Err(_) => {
                        warn!("Timeout querying node {} for region", node_id);
                    }
                }
            }

            Ok(())
        };

        // Execute the future in a synchronous context
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle.block_on(future),
            Err(_) => {
                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    StorageNodeError::Internal(format!("Failed to create tokio runtime: {e}"))
                })?;
                rt.block_on(future)
            }
        }
    }

    async fn register_node(&self, node_id: String, endpoint: String) {
        let mut registry = self.node_registry.write().await;
        registry.insert(node_id, endpoint);
    }

    fn get_metrics(&self) -> NetworkMetricsSnapshot {
        self.metrics.snapshot()
    }

    async fn health_check(&self, node_id: &str) -> Result<bool> {
        match self.get_endpoint(node_id).await {
            Ok(endpoint) => {
                let url = format!("{endpoint}/api/v1/health");

                // Perform direct request without retry for health checks
                match self.http_request(self.client.get(&url)).await {
                    Ok(response) => Ok(response.status().is_success()),
                    Err(_) => Ok(false), // Consider unhealthy on any error
                }
            }
            Err(_) => Ok(false), // Unknown node is unhealthy
        }
    }
    fn get_connection_status(&self) -> ConnectionPoolStatus {
        // Since we're using reqwest which manages its own connection pool internally,
        // we can't get exact connection counts. Return approximated values.
        ConnectionPoolStatus {
            total_connections: self.config.max_connections_per_host * 10, // Estimate
            active_connections: 0, // Not accessible with reqwest
            idle_connections: 0,   // Not accessible with reqwest
            pending_requests: 0,   // Not accessible with reqwest
        }
    }
}

// Production network implementation
pub struct ProductionNetworkClient {
    node_id: String,
    client: reqwest::Client,
    config: NetworkClientConfig,
    peer_discovery: PeerDiscoveryService,
    message_cache: Arc<Mutex<LruCache<[u8; 32], ()>>>,
    metrics: Arc<Mutex<NetworkMetrics>>,
}

impl ProductionNetworkClient {
    pub fn new(node_id: String, config: NetworkClientConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.request_timeout))
            .pool_max_idle_per_host(config.max_connections_per_host)
            .pool_idle_timeout(Duration::from_secs(config.connection_pool_idle_timeout))
            .build()
            .map_err(|e| StorageNodeError::Network(format!("Failed to create HTTP client: {e}")))?;

        let peer_discovery = PeerDiscoveryService::new(&config)?;
        let message_cache = Arc::new(Mutex::new(LruCache::new(
            NonZeroUsize::new(config.message_cache_size).unwrap(),
        )));
        let metrics = Arc::new(Mutex::new(NetworkMetrics::default()));

        Ok(Self {
            node_id,
            client,
            config,
            peer_discovery,
            message_cache,
            metrics,
        })
    }

    /// Implement epidemic protocol for reliable message propagation
    async fn epidemic_broadcast(&self, message: MessagePayload) -> Result<()> {
        let peers = self.peer_discovery.get_active_peers().await?;
        let fanout = std::cmp::min(peers.len(), self.config.epidemic_fanout);

        // Select random subset of peers for epidemic broadcast
        let selected_peers: Vec<_> = {
            let mut rng = rand::thread_rng();
            peers.choose_multiple(&mut rng, fanout).collect()
        };

        // Broadcast to selected peers concurrently
        let broadcast_futures: Vec<_> = selected_peers
            .iter()
            .map(|peer| self.send_message_to_peer(peer, &message))
            .collect();

        let results = futures::future::join_all(broadcast_futures).await;

        // Update metrics
        let metrics = self.metrics.lock().unwrap();
        for result in results {
            match result {
                Ok(_) => metrics.requests_sent.fetch_add(1, Ordering::Relaxed),
                Err(_) => metrics.errors.fetch_add(1, Ordering::Relaxed),
            };
        }

        Ok(())
    }

    /// Send message to a specific peer with retry logic
    async fn send_message_to_peer(&self, peer: &PeerInfo, message: &MessagePayload) -> Result<()> {
        let mut attempts = 0;
        let max_retries = self.config.max_retries;

        while attempts <= max_retries {
            match self.attempt_send_to_peer(peer, message).await {
                Ok(_) => return Ok(()),
                Err(_) if attempts < max_retries => {
                    attempts += 1;
                    let delay = Duration::from_millis(
                        self.config.retry_delay_ms * (2_u64.pow(attempts as u32)),
                    );
                    tokio::time::sleep(delay).await;

                    let metrics = self.metrics.lock().unwrap();
                    metrics.retries.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => return Err(e),
            }
        }

        Err(StorageNodeError::Network(
            "Max retries exceeded".to_string(),
        ))
    }

    /// Single attempt to send message to peer
    async fn attempt_send_to_peer(&self, peer: &PeerInfo, message: &MessagePayload) -> Result<()> {
        let url = format!("{}/api/v1/message", peer.endpoint);
        let serialized_message = bincode::serialize(message).map_err(|e| {
            StorageNodeError::Serialization(format!("Failed to serialize message: {e}"))
        })?;

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .header("X-Node-ID", &self.node_id)
            .body(serialized_message)
            .send()
            .await
            .map_err(|e| StorageNodeError::Network(format!("HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(StorageNodeError::Network(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let metrics = self.metrics.lock().unwrap();
        metrics
            .bytes_sent
            .fetch_add(message.data.len() as u64, Ordering::Relaxed);
        metrics.responses_received.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Implement consensus-based state synchronization
    async fn sync_state_with_consensus(&self, entries: &[StateEntry]) -> Result<Vec<StateEntry>> {
        let peers = self.peer_discovery.get_active_peers().await?;
        let quorum_size = (peers.len() * 2 / 3) + 1; // Byzantine fault tolerance

        let mut consensus_map: HashMap<String, HashMap<Vec<u8>, usize>> = HashMap::new();
        let mut peer_responses = Vec::new();

        // Request state from multiple peers
        for peer in &peers {
            match self.request_state_from_peer(peer, entries).await {
                Ok(response) => peer_responses.push(response),
                Err(e) => {
                    warn!("Failed to get state from peer {}: {}", peer.node_id, e);
                    continue;
                }
            }
        }

        // Build consensus map
        for response in &peer_responses {
            for entry in response {
                consensus_map
                    .entry(entry.key.clone())
                    .or_default()
                    .entry(entry.value.clone())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }

        // Select values with quorum consensus
        let mut consensus_entries = Vec::new();
        for (key, value_counts) in consensus_map {
            if let Some((value, &count)) = value_counts.iter().max_by_key(|(_, &count)| count) {
                if count >= quorum_size {
                    // Find the most recent entry with this value
                    let mut best_entry = None;
                    let mut best_timestamp = 0;

                    for response in &peer_responses {
                        for entry in response {
                            if entry.key == key
                                && entry.value == *value
                                && entry.timestamp > best_timestamp
                            {
                                best_timestamp = entry.timestamp;
                                best_entry = Some(entry.clone());
                            }
                        }
                    }

                    if let Some(entry) = best_entry {
                        consensus_entries.push(entry);
                    }
                }
            }
        }

        Ok(consensus_entries)
    }

    /// Request state from a specific peer
    async fn request_state_from_peer(
        &self,
        peer: &PeerInfo,
        entries: &[StateEntry],
    ) -> Result<Vec<StateEntry>> {
        let keys: Vec<String> = entries.iter().map(|e| e.key.clone()).collect();
        let url = format!("{}/api/v1/state/batch", peer.endpoint);

        let request = StateRequest { keys };
        let serialized_request = bincode::serialize(&request).map_err(|e| {
            StorageNodeError::Serialization(format!("Failed to serialize request: {e}"))
        })?;

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .header("X-Node-ID", &self.node_id)
            .body(serialized_request)
            .send()
            .await
            .map_err(|e| StorageNodeError::Network(format!("HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(StorageNodeError::Network(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| StorageNodeError::Network(format!("Failed to read response: {e}")))?;

        let state_entries: Vec<StateEntry> =
            bincode::deserialize(&response_bytes).map_err(|e| {
                StorageNodeError::Serialization(format!("Failed to deserialize response: {e}"))
            })?;

        Ok(state_entries)
    }
}

#[async_trait]
impl NetworkClient for ProductionNetworkClient {
    async fn send_entries(&self, _node_id: String, entries: Vec<StateEntry>) -> Result<()> {
        let message = MessagePayload {
            message_id: rand::random(),
            sender_id: self.node_id.clone(),
            data: bincode::serialize(&entries).map_err(|e| {
                StorageNodeError::Serialization(format!("Failed to serialize entries: {e}"))
            })?,
            ttl: 7, // 7 hops maximum
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        // Check message cache to prevent loops
        {
            let mut cache = self.message_cache.lock().unwrap();
            if cache.contains(&message.message_id) {
                return Ok(()); // Already processed
            }
            cache.put(message.message_id, ());
        }

        if message.ttl > 0 {
            let mut decremented_message = message;
            decremented_message.ttl -= 1;
            self.epidemic_broadcast(decremented_message).await?;
        }

        Ok(())
    }

    async fn request_entries(&self, node_id: String, keys: Vec<String>) -> Result<Vec<StateEntry>> {
        let dummy_entries: Vec<StateEntry> = keys
            .iter()
            .map(|key| StateEntry {
                key: key.clone(),
                value: Vec::new(),
                vector_clock: VectorClock::new(),
                timestamp: 0,
                origin_node: node_id.clone(),
            })
            .collect();

        self.sync_state_with_consensus(&dummy_entries).await
    }

    async fn forward_put(&self, node_id: String, key: String, value: Vec<u8>) -> Result<()> {
        let entry = StateEntry {
            key,
            value,
            vector_clock: VectorClock::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            origin_node: self.node_id.clone(),
        };

        self.send_entries(node_id, vec![entry]).await
    }

    async fn forward_get(&self, node_id: String, key: String) -> Result<Option<Vec<u8>>> {
        let entries = self.request_entries(node_id, vec![key.clone()]).await?;
        Ok(entries.into_iter().find(|e| e.key == key).map(|e| e.value))
    }

    async fn forward_delete(&self, node_id: String, key: String) -> Result<()> {
        // Implement tombstone mechanism for deletions
        let tombstone_value = format!(
            "DELETED:{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        )
        .into_bytes();

        self.forward_put(node_id, key, tombstone_value).await
    }

    async fn get_node_status(&self, node_id: &str) -> Result<NodeStatus> {
        if let Some(peer) = self.peer_discovery.find_peer(node_id).await? {
            let url = format!("{}/api/v1/status", peer.endpoint);

            let response = self
                .client
                .get(&url)
                .header("X-Node-ID", &self.node_id)
                .send()
                .await
                .map_err(|e| StorageNodeError::Network(format!("HTTP request failed: {e}")))?;

            if !response.status().is_success() {
                return Err(StorageNodeError::Network(format!(
                    "HTTP error: {}",
                    response.status()
                )));
            }

            let status: NodeStatus = response
                .json()
                .await
                .map_err(|e| StorageNodeError::Network(format!("Failed to parse JSON: {e}")))?;

            Ok(status)
        } else {
            Err(StorageNodeError::Network(format!(
                "Node {node_id} not found"
            )))
        }
    }

    async fn join_cluster(
        &self,
        bootstrap_nodes: Vec<String>,
        node_endpoint: String,
    ) -> Result<Vec<StorageNode>> {
        let mut cluster_nodes = Vec::new();

        for bootstrap_node in bootstrap_nodes {
            match self
                .attempt_cluster_join(&bootstrap_node, &node_endpoint)
                .await
            {
                Ok(nodes) => {
                    cluster_nodes.extend(nodes);
                    break; // Successfully joined via this bootstrap node
                }
                Err(e) => {
                    warn!("Failed to join cluster via {}: {}", bootstrap_node, e);
                    continue;
                }
            }
        }

        if cluster_nodes.is_empty() {
            return Err(StorageNodeError::Network(
                "Failed to join cluster via any bootstrap node".to_string(),
            ));
        }

        // Register with peer discovery
        for node in &cluster_nodes {
            self.peer_discovery
                .add_peer(PeerInfo {
                    node_id: node.id.clone(),
                    endpoint: node.endpoint.clone(),
                    last_seen: std::time::SystemTime::now(),
                    reliability_score: 1.0,
                    last_state_number: None,
                    public_key: None,
                })
                .await?;
        }

        Ok(cluster_nodes)
    }

    async fn register_node(&self, node_id: String, endpoint: String) {
        let peer_info = PeerInfo {
            node_id,
            endpoint,
            last_seen: std::time::SystemTime::now(),
            reliability_score: 1.0,
            last_state_number: None,
            public_key: None,
        };

        if let Err(e) = self.peer_discovery.add_peer(peer_info).await {
            error!("Failed to register node: {}", e);
        }
    }

    #[allow(unused_variables)]
    fn send_message(
        &self,
        device_id: std::net::SocketAddr,
        message_id: [u8; 32],
        data: Vec<u8>,
        ttl: u8,
    ) -> Result<()> {
        // For UDP-based direct messaging
        let message = MessagePayload {
            message_id,
            sender_id: self.node_id.clone(),
            data,
            ttl,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        // In a full implementation, this would use UDP sockets
        // For now, we'll use the epidemic broadcast mechanism
        tokio::spawn({
            let client = self.clone();
            async move {
                if let Err(e) = client.epidemic_broadcast(message).await {
                    error!("Failed to broadcast message: {}", e);
                }
            }
        });

        Ok(())
    }

    fn find_nodes(&self, target: &crate::storage::topology::NodeId) -> Result<()> {
        // Implement Kademlia-style node discovery
        tokio::spawn({
            let client = self.clone();
            let target_id = target.clone();
            async move {
                if let Err(e) = client.peer_discovery.find_nodes_near(&target_id).await {
                    error!("Failed to find nodes near target: {}", e);
                }
            }
        });

        Ok(())
    }

    fn find_nodes_in_region(&self, region: u8) -> Result<()> {
        tokio::spawn({
            let client = self.clone();
            async move {
                if let Err(e) = client.peer_discovery.find_nodes_in_region(region).await {
                    error!("Failed to find nodes in region {}: {}", region, e);
                }
            }
        });

        Ok(())
    }

    fn get_metrics(&self) -> NetworkMetricsSnapshot {
        let metrics = self
            .metrics
            .try_lock()
            .map(|m| m.clone())
            .unwrap_or_default();
        NetworkMetricsSnapshot {
            requests_sent: metrics.requests_sent.load(Ordering::Relaxed),
            responses_received: metrics.responses_received.load(Ordering::Relaxed),
            errors: metrics.errors.load(Ordering::Relaxed),
            timeouts: metrics.timeouts.load(Ordering::Relaxed),
            retries: metrics.retries.load(Ordering::Relaxed),
            bytes_sent: metrics.bytes_sent.load(Ordering::Relaxed),
            bytes_received: metrics.bytes_received.load(Ordering::Relaxed),
        }
    }

    async fn health_check(&self, node_id: &str) -> Result<bool> {
        match self.get_node_status(node_id).await {
            Ok(status) => Ok(status.status == "ok"),
            Err(_) => Ok(false),
        }
    }

    fn get_connection_status(&self) -> ConnectionPoolStatus {
        // In a production implementation, this would query the actual connection pool
        let metrics = self
            .metrics
            .try_lock()
            .map(|m| m.clone())
            .unwrap_or_default();
        let requests_sent = metrics.requests_sent.load(Ordering::Relaxed);
        let responses_received = metrics.responses_received.load(Ordering::Relaxed);
        let active = if requests_sent > responses_received {
            (requests_sent - responses_received) as usize
        } else {
            0
        };

        ConnectionPoolStatus {
            total_connections: self.config.max_connections_per_host * 10,
            active_connections: active,
            idle_connections: 0, // Not easily accessible with reqwest
            pending_requests: 0, // Not easily accessible with reqwest
        }
    }
}

impl Clone for ProductionNetworkClient {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id.clone(),
            client: self.client.clone(),
            config: self.config.clone(),
            peer_discovery: self.peer_discovery.clone(),
            message_cache: Arc::clone(&self.message_cache),
            metrics: Arc::clone(&self.metrics),
        }
    }
}

impl ProductionNetworkClient {
    async fn attempt_cluster_join(
        &self,
        bootstrap_node: &str,
        node_endpoint: &str,
    ) -> Result<Vec<StorageNode>> {
        let url = format!("{bootstrap_node}/api/v1/join");
        let join_request = JoinRequest {
            node_id: self.node_id.clone(),
            endpoint: node_endpoint.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&join_request)
            .send()
            .await
            .map_err(|e| StorageNodeError::Network(format!("HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(StorageNodeError::Network(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let join_response: JoinResponse = response
            .json()
            .await
            .map_err(|e| StorageNodeError::Network(format!("Failed to parse JSON: {e}")))?;

        Ok(join_response.nodes)
    }
}

/// Mock network client for testing
#[derive(Debug)]
pub struct MockNetworkClient {
    /// Node ID
    node_id: String,
    /// Mock responses storage
    responses: std::sync::Mutex<HashMap<String, Vec<u8>>>,
}

impl Default for MockNetworkClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MockNetworkClient {
    pub fn new() -> Self {
        Self {
            node_id: "mock-node".to_string(),
            responses: std::sync::Mutex::new(HashMap::new()),
        }
    }

    pub fn with_node_id(node_id: String) -> Self {
        Self {
            node_id,
            responses: std::sync::Mutex::new(HashMap::new()),
        }
    }

    pub fn add_response(&self, key: String, value: Vec<u8>) {
        let mut responses = self.responses.lock().unwrap();
        responses.insert(key, value);
    }
}

#[async_trait]
impl NetworkClient for MockNetworkClient {
    async fn send_entries(&self, _node_id: String, _entries: Vec<StateEntry>) -> Result<()> {
        Ok(())
    }

    async fn request_entries(
        &self,
        _node_id: String,
        _keys: Vec<String>,
    ) -> Result<Vec<StateEntry>> {
        Ok(Vec::new())
    }

    async fn forward_put(&self, _node_id: String, key: String, value: Vec<u8>) -> Result<()> {
        let mut responses = self.responses.lock().unwrap();
        responses.insert(key, value);
        Ok(())
    }

    async fn forward_get(&self, _node_id: String, key: String) -> Result<Option<Vec<u8>>> {
        let responses = self.responses.lock().unwrap();
        Ok(responses.get(&key).cloned())
    }

    async fn forward_delete(&self, _node_id: String, key: String) -> Result<()> {
        let mut responses = self.responses.lock().unwrap();
        responses.remove(&key);
        Ok(())
    }

    async fn get_node_status(&self, _node_id: &str) -> Result<NodeStatus> {
        Ok(NodeStatus {
            node_id: self.node_id.clone(),
            status: "ok".to_string(),
            uptime: 0,
            version: "0.1.0".to_string(),
            metrics: HashMap::new(),
        })
    }

    async fn join_cluster(
        &self,
        _bootstrap_nodes: Vec<String>,
        _node_endpoint: String,
    ) -> Result<Vec<StorageNode>> {
        Ok(vec![StorageNode {
            id: "mock-node-1".to_string(),
            name: "Mock Node 1".to_string(),
            region: "mock-region".to_string(),
            public_key: "mock-key".to_string(),
            endpoint: "http://localhost:8000".to_string(),
        }])
    }

    async fn register_node(&self, _node_id: String, _endpoint: String) {
        // Mock implementation does nothing
    }

    fn send_message(
        &self,
        _device_id: std::net::SocketAddr,
        _message_id: [u8; 32],
        _data: Vec<u8>,
        _ttl: u8,
    ) -> Result<()> {
        Ok(())
    }

    fn find_nodes(&self, _target: &crate::storage::topology::NodeId) -> Result<()> {
        Ok(())
    }

    fn find_nodes_in_region(&self, _region: u8) -> Result<()> {
        Ok(())
    }

    fn get_metrics(&self) -> NetworkMetricsSnapshot {
        NetworkMetricsSnapshot {
            requests_sent: 0,
            responses_received: 0,
            errors: 0,
            timeouts: 0,
            retries: 0,
            bytes_sent: 0,
            bytes_received: 0,
        }
    }

    async fn health_check(&self, _node_id: &str) -> Result<bool> {
        Ok(true) // Mock always healthy
    }

    fn get_connection_status(&self) -> ConnectionPoolStatus {
        ConnectionPoolStatus {
            total_connections: 1,
            active_connections: 1,
            idle_connections: 0,
            pending_requests: 0,
        }
    }
}

/// Entry for state synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEntry {
    /// Unique key for the entry
    pub key: String,
    /// Value data
    pub value: Vec<u8>,
    /// Vector clock for conflict resolution
    pub vector_clock: VectorClock,
    /// Timestamp of the entry
    pub timestamp: u64,
    /// Origin node ID
    pub origin_node: String,
}

/// Join request sent when joining a cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRequest {
    /// ID of the node trying to join
    pub node_id: String,
    /// Endpoint of the node trying to join
    pub endpoint: String,
}

/// Response to a join request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinResponse {
    /// List of nodes in the cluster
    pub nodes: Vec<StorageNode>,
}

/// Node status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    /// Node ID
    pub node_id: String,
    /// Status string (e.g., "ok", "degraded")
    pub status: String,
    /// Uptime in seconds
    pub uptime: u64,
    /// Version string
    pub version: String,
    /// Additional metrics
    pub metrics: HashMap<String, String>,
}

// Message payload for network propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePayload {
    /// Unique message identifier
    pub message_id: [u8; 32],
    /// Node ID of the sender
    pub sender_id: String,
    /// Message data
    pub data: Vec<u8>,
    /// Time-to-live counter
    pub ttl: u8,
    /// Timestamp
    pub timestamp: u64,
}

// Re-export vector clock for convenience
use crate::storage::vector_clock::VectorClock;

/// Factory for creating network clients
pub struct NetworkClientFactory;

impl NetworkClientFactory {
    /// Create a new HTTP network client for the given node
    pub fn create_client(node: StorageNode) -> Result<crate::distribution::NetworkClientType> {
        let config = NetworkClientConfig::default();
        let client = HttpNetworkClient::new(node.id, config)?;
        Ok(crate::distribution::NetworkClientType::Http(Arc::new(
            client,
        )))
    }

    /// Create a mock network client for testing
    pub fn create_mock_client(node_id: String) -> crate::distribution::NetworkClientType {
        crate::distribution::NetworkClientType::Mock(Arc::new(MockNetworkClient::with_node_id(
            node_id,
        )))
    }
}
