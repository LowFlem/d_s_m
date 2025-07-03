#!/bin/bash

echo "🌐 DSM Storage Node Cluster Status Check"
echo "========================================"

# Function to check if a port is in use
check_port() {
    local port=$1
    local node_name=$2
    
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null; then
        echo "✓ $node_name (port $port): Running"
        return 0
    else
        echo "✗ $node_name (port $port): Not running"
        return 1
    fi
}

# Function to check node health via API
check_health() {
    local port=$1
    local node_name=$2
    
    local response=$(curl -s -f http://localhost:$port/api/v1/health 2>/dev/null)
    if [ $? -eq 0 ]; then
        echo "  🏥 Health check: ✓ OK"
        # Parse and show additional info
        local status=$(echo "$response" | jq -r '.status // "unknown"' 2>/dev/null)
        echo "  📊 Status: $status"
    else
        echo "  🏥 Health check: ✗ Failed"
    fi
}

# Function to check node status via API
check_status() {
    local port=$1
    local node_name=$2
    
    local response=$(curl -s -f http://localhost:$port/api/v1/status 2>/dev/null)
    if [ $? -eq 0 ]; then
        echo "  📈 Node status: ✓ Available"
        # Parse and show node info
        local node_id=$(echo "$response" | jq -r '.node_id // "unknown"' 2>/dev/null)
        local peers=$(echo "$response" | jq -r '.peers // 0' 2>/dev/null)
        echo "  🆔 Node ID: $node_id"
        echo "  🤝 Peers: $peers"
    else
        echo "  📈 Node status: ✗ Unavailable"
    fi
}

running_count=0
total_nodes=5

echo
echo "Checking individual nodes..."
echo "----------------------------"

# Check each node
for i in {1..5}; do
    port=$((8079 + i))
    node_name="Node $i"
    
    echo
    echo "🔍 Checking $node_name (http://localhost:$port)"
    
    if check_port $port "$node_name"; then
        ((running_count++))
        check_health $port "$node_name"
        check_status $port "$node_name"
    fi
done

echo
echo "=================================="
echo "📊 Cluster Summary"
echo "=================================="
echo "Running nodes: $running_count/$total_nodes"

if [ $running_count -eq $total_nodes ]; then
    echo "🎉 All nodes are running!"
    exit 0
elif [ $running_count -gt 0 ]; then
    echo "⚠️  Some nodes are not running"
    exit 1
else
    echo "❌ No nodes are running"
    exit 2
fi
