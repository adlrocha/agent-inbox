#!/bin/bash
# Test parallel agent execution

export PATH="$HOME/.local/bin:$(dirname "$0"):$PATH"

echo "Starting 3 parallel agents..."

# Start 3 agents in parallel
./wrappers/test-agent-wrapper "task 1" &
PID1=$!

./wrappers/test-agent-wrapper "task 2" &
PID2=$!

./wrappers/test-agent-wrapper "task 3" &
PID3=$!

# Give them a moment to register
sleep 1

echo ""
echo "Tasks running, checking agent-inbox:"
agent-inbox list --all

# Wait for completion
wait $PID1 $PID2 $PID3

echo ""
echo "All tasks completed, checking final state:"
sleep 1
agent-inbox list --all
