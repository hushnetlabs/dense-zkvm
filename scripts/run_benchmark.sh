#!/bin/bash

echo "Starting Mock Prover Servers for Benchmark Testing"
echo "===================================================="

# Function to start a mock server on a given port
start_mock_server() {
    local port=$1
    PORT=$port cargo run --bin densezk-server --features server --release &
    echo "Started server on port $port (PID: $!)"
}

# Kill any existing servers
pkill -f "densezk-server" 2>/dev/null || true
sleep 1

# Start multiple mock server instances (simulating prover network)
echo ""
echo "Starting 4 mock prover servers on ports 8080, 8081, 8082, 8083..."
echo ""

start_mock_server 8080 &
start_mock_server 8081 &
start_mock_server 8082 &
start_mock_server 8083 &

sleep 2

echo ""
echo "Servers started. Running benchmark..."
echo ""

# Run the benchmark
SERVER_URL="http://localhost:8080" cargo run --bin network-benchmark --features sync --release

echo ""
echo "Benchmark complete."
echo ""
echo "To stop servers: pkill -f 'densezk-server'"