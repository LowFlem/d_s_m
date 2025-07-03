//! Policy API for DSM Storage Node
//!
//! This module provides the HTTP API endpoints for storing, retrieving,
//! and managing token policies on storage nodes.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::error::{Result, StorageNodeError};
use crate::policy::{
    GetPolicyRequest, GetPolicyResponse, PolicyStore, StorePolicyRequest, StorePolicyResponse,
};

/// Storage Node State that includes PolicyStore
#[derive(Clone)]
pub struct ApiState {
    policy_store: Arc<PolicyStore>,
}

impl ApiState {
    pub fn new(policy_store: Arc<PolicyStore>) -> Self {
        Self { policy_store }
    }
}

/// Store a policy
pub async fn store_policy(
    State(state): State<ApiState>,
    Json(req): Json<StorePolicyRequest>,
) -> Result<Json<StorePolicyResponse>> {
    info!("API: Store policy request received");

    let response = state.policy_store.store_policy(req).await?;
    Ok(Json(response))
}

/// Get a policy by ID
pub async fn get_policy(
    State(state): State<ApiState>,
    Path(policy_id): Path<String>,
) -> Result<Json<GetPolicyResponse>> {
    debug!("API: Get policy request for ID: {}", policy_id);

    let req = GetPolicyRequest { policy_id };
    let response = state.policy_store.get_policy(req).await?;
    Ok(Json(response))
}

/// List all policies
#[derive(Serialize)]
pub struct ListPoliciesResponse {
    pub policies: Vec<String>,
    pub count: usize,
}

pub async fn list_policies(
    State(state): State<ApiState>,
) -> Result<Json<ListPoliciesResponse>> {
    debug!("API: List policies request");

    let policies = state.policy_store.list_policies(None, None).await?;
    let response = ListPoliciesResponse {
        count: policies.len(),
        policies,
    };
    
    Ok(Json(response))
}

/// Delete a policy (admin only)
#[derive(Serialize)]
pub struct DeletePolicyResponse {
    pub policy_id: String,
    pub success: bool,
}

pub async fn delete_policy(
    State(state): State<ApiState>,
    Path(policy_id): Path<String>,
) -> Result<Json<DeletePolicyResponse>> {
    info!("API: Delete policy request for ID: {}", policy_id);

    let success = state.policy_store.remove_policy(&policy_id).await?;
    let response = DeletePolicyResponse {
        policy_id,
        success,
    };
    
    Ok(Json(response))
}
