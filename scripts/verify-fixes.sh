#!/bin/bash

# DSM Technical Review Fix Verification Script
# Validates that all critical fixes from the technical review have been implemented

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_failure() {
    echo -e "${RED}[✗]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

check_item() {
    TOTAL_CHECKS=$((TOTAL_CHECKS+1))
    if [ $1 -eq 0 ]; then
        log_success "$2"
        PASSED_CHECKS=$((PASSED_CHECKS+1))
    else
        log_failure "$2"
        FAILED_CHECKS=$((FAILED_CHECKS+1))
    fi
}
echo "========================================="
echo "DSM Technical Review Fix Verification"
echo "========================================="
echo

# Critical Finding A1: Configuration Consolidation
log_info "Checking A1: Configuration Consolidation"

# Check for single consolidated Cargo.toml
if [ -f "${PROJECT_ROOT}/dsm_client/decentralized_state_machine/dsm_sdk/Cargo.toml" ]; then
    # Verify it has profile-based configuration
    if grep -q "\[profile\.minimal\]" "${PROJECT_ROOT}/dsm_client/decentralized_state_machine/dsm_sdk/Cargo.toml"; then
        check_item 0 "Consolidated Cargo.toml with profile-based configuration exists"
    else
        check_item 1 "Cargo.toml exists but missing profile configuration"
    fi
else
    check_item 1 "Consolidated Cargo.toml missing"
fi

# Check for removal of duplicate configs (should not exist)
duplicate_configs=(
    "Cargo-fixed.toml"
    "Cargo-diagnostic.toml" 
    "Cargo-minimal.toml"
)

for config in "${duplicate_configs[@]}"; do
    if [ -f "${PROJECT_ROOT}/dsm_client/decentralized_state_machine/dsm_sdk/${config}" ]; then
        check_item 1 "Duplicate config ${config} still exists (should be removed)"
    else
        check_item 0 "Duplicate config ${config} properly removed"
    fi
done

echo

# Critical Finding A4: Proto Definitions
log_info "Checking A4: Protobuf Definitions"

if [ -f "${PROJECT_ROOT}/proto/dsm_app.proto" ]; then
    # Check for key message types
    if grep -q "message GenesisRequest" "${PROJECT_ROOT}/proto/dsm_app.proto" && \
       grep -q "message TransferRequest" "${PROJECT_ROOT}/proto/dsm_app.proto" && \
       grep -q "message Envelope" "${PROJECT_ROOT}/proto/dsm_app.proto"; then
        check_item 0 "Complete protobuf definitions with all required messages"
    else
        check_item 1 "Protobuf file exists but missing required messages"
    fi
    
    # Check for forward compatibility
    if grep -q "oneof payload" "${PROJECT_ROOT}/proto/dsm_app.proto"; then
        check_item 0 "Forward-compatible Envelope structure implemented"
    else
        check_item 1 "Missing forward-compatible Envelope structure"
    fi
else
    check_item 1 "Canonical dsm_app.proto file missing"
fi

echo

# Critical Finding A3: Empty Directory Handling
log_info "Checking A3: Empty Directory Handling"

# Check that bluetooth directory is properly stubbed
if [ -f "${PROJECT_ROOT}/dsm_client/decentralized_state_machine/dsm/src/bilateral/bluetooth/README.md" ] && \
   [ -f "${PROJECT_ROOT}/dsm_client/decentralized_state_machine/dsm/src/bilateral/bluetooth/mod.rs" ]; then
    check_item 0 "Bluetooth directory properly stubbed with documentation"
else
    check_item 1 "Bluetooth directory not properly stubbed"
fi

echo

# Critical Finding A6: Security Documentation
log_info "Checking A6: Formal Security Documentation"

if [ -f "${PROJECT_ROOT}/docs/SECURITY.md" ]; then
    # Check for key sections
    if grep -q "STRIDE Analysis" "${PROJECT_ROOT}/docs/SECURITY.md" && \
       grep -q "Post-Quantum" "${PROJECT_ROOT}/docs/SECURITY.md" && \
       grep -q "Threat Model" "${PROJECT_ROOT}/docs/SECURITY.md"; then
        check_item 0 "Comprehensive security documentation with threat modeling"
    else
        check_item 1 "Security documentation exists but incomplete"
    fi
else
    check_item 1 "Formal security documentation missing"
fi

echo

# SBOM and Compliance
log_info "Checking SBOM and Compliance Features"

if [ -f "${PROJECT_ROOT}/scripts/generate-sbom.sh" ] && [ -x "${PROJECT_ROOT}/scripts/generate-sbom.sh" ]; then
    check_item 0 "SBOM generation script exists and is executable"
else
    check_item 1 "SBOM generation script missing or not executable"
fi

echo

# Documentation Updates
log_info "Checking Documentation Updates"

if [ -f "${PROJECT_ROOT}/AGENT.md" ]; then
    if grep -q "generate-sbom.sh" "${PROJECT_ROOT}/AGENT.md" && \
       grep -q "docs/SECURITY.md" "${PROJECT_ROOT}/AGENT.md"; then
        check_item 0 "AGENT.md updated with new security and SBOM commands"
    else
        check_item 1 "AGENT.md not updated with new features"
    fi
else
    check_item 1 "AGENT.md file missing"
fi

echo

# Architecture Compliance
log_info "Checking Architecture Compliance"

# Check for proper feature flags in consolidated config
if [ -f "${PROJECT_ROOT}/dsm_client/decentralized_state_machine/dsm_sdk/Cargo.toml" ]; then
    if grep -q 'full = \[.*"networking"' "${PROJECT_ROOT}/dsm_client/decentralized_state_machine/dsm_sdk/Cargo.toml" && \
       grep -q 'minimal = \[.*"bluetooth"' "${PROJECT_ROOT}/dsm_client/decentralized_state_machine/dsm_sdk/Cargo.toml" && \
       grep -q '\[features\]' "${PROJECT_ROOT}/dsm_client/decentralized_state_machine/dsm_sdk/Cargo.toml"; then
        check_item 0 "Feature-based configuration properly implemented"
    else
        check_item 1 "Feature-based configuration incomplete"
    fi
fi

# Check for post-quantum crypto dependencies
if grep -q "ml-kem" "${PROJECT_ROOT}/dsm_client/decentralized_state_machine/dsm_sdk/Cargo.toml"; then
    check_item 0 "Post-quantum cryptography dependencies present"
else
    check_item 1 "Post-quantum cryptography dependencies missing"
fi

echo

# Build System Integrity
log_info "Checking Build System Integrity"

# Verify key build files exist
build_files=(
    "dsm_client/decentralized_state_machine/Cargo.toml"
    "dsm_storage_node/Cargo.toml"
    "dsm_client/new_frontend/package.json"
)

for build_file in "${build_files[@]}"; do
    if [ -f "${PROJECT_ROOT}/${build_file}" ]; then
        check_item 0 "Build file ${build_file} exists"
    else
        check_item 1 "Build file ${build_file} missing"
    fi
done

echo

# Summary
echo "========================================="
echo "VERIFICATION SUMMARY"
echo "========================================="
echo "Total Checks: ${TOTAL_CHECKS}"
echo "Passed: ${PASSED_CHECKS}"
echo "Failed: ${FAILED_CHECKS}"
echo

if [ ${FAILED_CHECKS} -eq 0 ]; then
    log_success "ALL CHECKS PASSED! Technical review fixes successfully implemented."
    echo
    echo "✅ Configuration consolidation complete"
    echo "✅ Protobuf definitions implemented" 
    echo "✅ Empty directories properly handled"
    echo "✅ Security documentation created"
    echo "✅ SBOM generation capability added"
    echo "✅ Architecture compliance maintained"
    echo
    exit 0
else
    log_failure "${FAILED_CHECKS} checks failed. Please address the issues above."
    echo
    echo "Next steps:"
    echo "1. Review failed checks above"
    echo "2. Fix any missing or incomplete items"
    echo "3. Re-run verification: ./scripts/verify-fixes.sh"
    echo
    exit 1
fi
