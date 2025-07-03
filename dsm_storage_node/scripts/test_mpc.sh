#!/bin/bash

echo "Testing MPC Contribution Endpoint"
echo "================================="

curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "test_session_123",
    "device_id": "test_device_456", 
    "contribution_type": "genesis",
    "timestamp": 1640995200
  }' \
  http://192.168.7.55:8080/api/v1/mpc/contribute

echo ""
echo ""
echo "Testing Health Endpoint"
echo "======================"

curl -s http://192.168.7.55:8080/health | head -c 200

echo ""
