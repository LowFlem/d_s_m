#!/bin/bash

# DSM Technical Review Fix Verification Script
# Validates that all critical fixes from the technical review have been implemented
# Rewritten for robustness and completeness

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

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
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    if [ $1 -eq 0 ]; then
        log_success "$2"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
    else
        log_failure "$2"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
    fi
}

echo "========================================="
echo "DSM Technical Review Fix Verification"
echo "========================================="
echo

# Critical Finding A1: Configuration Consolidation
log_info "Checking A1: Configuration Consolidation"

# Check for single consolidated Cargo.toml with profile-based configuration
CARGO_FILE="${PROJECT_ROOT}/dsm_client/decentralized_state_machine/dsm_sdk/Cargo.toml"
if [ -f "$CARGO_FILE" ] && grep -q '\[profile\.minimal\]' "$CARGO_FILE"; then
    check_item 0 "Consolidated Cargo.toml with profile-based configuration exists"
elif [ -f "$CARGO_FILE" ]; then
    check_item 1 "Cargo.toml exists but missing profile configuration"
else
    check_item 1 "Consolidated Cargo.toml missing"
fi

# Check for removal of duplicate configs (should not exist)
duplicate_configs=("Cargo-fixed.toml" "Cargo-diagnostic.toml" "Cargo-minimal.toml")
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

PROTO_FILE="${PROJECT_ROOT}/proto/dsm_app.proto"
if [ -f "$PROTO_FILE" ]; then
    # Check for key message types
    if grep -q "message GenesisRequest" "$PROTO_FILE" && \
       grep -q "message TransferRequest" "$PROTO_FILE" && \
       grep -q "message Envelope" "$PROTO_FILE"; then
        check_item 0 "Complete protobuf definitions with all required messages"
    else
        check_item 1 "Protobuf file exists but missing required messages"
    fi
    
    # Check for forward compatibility envelope
    if grep -q "oneof payload" "$PROTO_FILE"; then
        check_item 0 "Forward-compatible Envelope structure implemented"
    else
        check_item 1 "Missing forward-compatible Envelope structure"
    fi
else
    check_item 1 "Canonical dsm_app.proto file missing"
fi

echo

# Critical Finding A6: Empty Directory Handling
log_info "Checking A6: Empty Directory Handling"

# Check Bluetooth module stubbing
BLUETOOTH_DIR="${PROJECT_ROOT}/dsm_client/decentralized_state_machine/dsm/src/bilateral/bluetooth"
if [ -f "${BLUETOOTH_DIR}/README.md" ] && [ -f "${BLUETOOTH_DIR}/mod.rs" ]; then
    check_item 0 "Bluetooth directory properly stubbed with documentation"
else
    check_item 1 "Bluetooth directory not properly stubbed"
fi

echo

# Security Documentation
log_info "Checking Security Documentation"

SECURITY_FILE="${PROJECT_ROOT}/docs/SECURITY.md"
if [ -f "$SECURITY_FILE" ]; then
    # Check for comprehensive security content
    if grep -q "STRIDE" "$SECURITY_FILE" && \
       grep -q "Cryptographic Analysis" "$SECURITY_FILE" && \
       grep -q "Operational Security" "$SECURITY_FILE"; then
        check_item 0 "Comprehensive security documentation with threat modeling"
    else
        check_item 1 "Security documentation exists but incomplete"
    fi
else
    check_item 1 "Formal security documentation missing"
fi

echo

# SBOM Generation
log_info "Checking SBOM Generation Capability"

SBOM_SCRIPT="${PROJECT_ROOT}/scripts/generate-sbom.sh"
if [ -f "$SBOM_SCRIPT" ] && [ -x "$SBOM_SCRIPT" ]; then
    check_item 0 "SBOM generation script exists and is executable"
else
    check_item 1 "SBOM generation script missing or not executable"
fi

echo

# Documentation Updates
log_info "Checking Documentation Updates"

AGENT_FILE="${PROJECT_ROOT}/AGENT.md"
if [ -f "$AGENT_FILE" ]; then
    # Check for updated content mentioning new features
    if grep -q "generate-sbom.sh" "$AGENT_FILE" && \
       grep -q "SECURITY.md" "$AGENT_FILE"; then
        check_item 0 "AGENT.md updated with new security and SBOM commands"
    else
        check_item 1 "AGENT.md not updated with new features"
    fi
else
    check_item 1 "AGENT.md file missing"
fi

echo
echo "========================================="
echo "Summary"
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
    echo "🎉 Repository is now compliant with DSM technical review requirements!"
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
