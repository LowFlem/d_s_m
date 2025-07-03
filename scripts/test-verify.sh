#!/bin/bash

# Simple test version of verification script
set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo "Project root: $PROJECT_ROOT"

# Test A1: Configuration Consolidation
echo "=== Testing A1: Configuration Consolidation ==="
CARGO_FILE="${PROJECT_ROOT}/dsm_client/decentralized_state_machine/dsm_sdk/Cargo.toml"
if [ -f "$CARGO_FILE" ]; then
    echo "✓ Cargo.toml exists"
    if grep -q '\[profile\.minimal\]' "$CARGO_FILE"; then
        echo "✓ Profile-based configuration found"
    else
        echo "✗ Profile-based configuration not found"
    fi
else
    echo "✗ Cargo.toml does not exist"
fi

# Test A4: Protobuf
echo "=== Testing A4: Protobuf ==="
PROTO_FILE="${PROJECT_ROOT}/proto/dsm_app.proto"
if [ -f "$PROTO_FILE" ]; then
    echo "✓ Protobuf file exists"
    if grep -q "message GenesisRequest" "$PROTO_FILE"; then
        echo "✓ GenesisRequest message found"
    else
        echo "✗ GenesisRequest message not found"
    fi
else
    echo "✗ Protobuf file does not exist"
fi

# Test A6: Empty directory handling
echo "=== Testing A6: Empty Directory Handling ==="
BLUETOOTH_DIR="${PROJECT_ROOT}/dsm_client/decentralized_state_machine/dsm/src/bilateral/bluetooth"
if [ -f "${BLUETOOTH_DIR}/README.md" ] && [ -f "${BLUETOOTH_DIR}/mod.rs" ]; then
    echo "✓ Bluetooth directory properly stubbed"
else
    echo "✗ Bluetooth directory not properly stubbed"
fi

# Test Security Documentation
echo "=== Testing Security Documentation ==="
if [ -f "${PROJECT_ROOT}/docs/SECURITY.md" ]; then
    echo "✓ Security documentation exists"
else
    echo "✗ Security documentation missing"
fi

# Test SBOM
echo "=== Testing SBOM Generation ==="
if [ -f "${PROJECT_ROOT}/scripts/generate-sbom.sh" ]; then
    echo "✓ SBOM generation script exists"
else
    echo "✗ SBOM generation script missing"
fi

echo "=== All tests completed ==="
