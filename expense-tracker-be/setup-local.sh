#!/bin/bash

# Setup script for local DynamoDB development
# This starts a local DynamoDB container and creates the table

TABLE_NAME="${DYNAMODB_TABLE:-expense_tracker}"
CONTAINER_NAME="dynamodb-local"

echo "=== Local DynamoDB Setup ==="
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "Error: Docker is not running. Please start Docker first."
    exit 1
fi

# Check if container already exists
if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "Container $CONTAINER_NAME already exists."
    
    # Check if it's running
    if docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        echo "Container is already running."
    else
        echo "Starting existing container..."
        docker start $CONTAINER_NAME
    fi
else
    echo "Creating new DynamoDB local container..."
    docker run -d \
        --name $CONTAINER_NAME \
        -p 8000:8000 \
        amazon/dynamodb-local
    
    echo "Waiting for DynamoDB to start..."
    sleep 3
fi

echo ""
echo "Creating table: $TABLE_NAME"

# Create table using local endpoint
aws dynamodb create-table \
    --table-name "$TABLE_NAME" \
    --attribute-definitions \
        AttributeName=pk,AttributeType=S \
        AttributeName=sk,AttributeType=S \
    --key-schema \
        AttributeName=pk,KeyType=HASH \
        AttributeName=sk,KeyType=RANGE \
    --billing-mode PAY_PER_REQUEST \
    --endpoint-url http://localhost:8000 \
    --region ap-south-1 2>&1

echo ""
echo "=== Setup Complete ==="
echo ""
echo "To run the server with local DynamoDB:"
echo "  export DYNAMODB_LOCAL=true"
echo "  export DYNAMODB_ENDPOINT=http://localhost:8000"
echo "  export DYNAMODB_TABLE=$TABLE_NAME"
echo "  cargo run"
echo ""
echo "To stop local DynamoDB:"
echo "  docker stop $CONTAINER_NAME"
echo ""
echo "To view table data:"
echo "  aws dynamodb scan --table-name $TABLE_NAME --endpoint-url http://localhost:8000 --region ap-south-1"
