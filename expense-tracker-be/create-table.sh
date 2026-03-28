#!/bin/bash

# Create DynamoDB table for expense tracker
# This script creates the table with the correct schema

TABLE_NAME="${DYNAMODB_TABLE:-expense_tracker}"
REGION="${AWS_REGION:-ap-south-1}"

echo "Creating DynamoDB table: $TABLE_NAME in region: $REGION"

aws dynamodb create-table \
    --table-name "$TABLE_NAME" \
    --attribute-definitions \
        AttributeName=pk,AttributeType=S \
        AttributeName=sk,AttributeType=S \
    --key-schema \
        AttributeName=pk,KeyType=HASH \
        AttributeName=sk,KeyType=RANGE \
    --billing-mode PAY_PER_REQUEST \
    --region "$REGION"

if [ $? -eq 0 ]; then
    echo "Table created successfully!"
    echo ""
    echo "Waiting for table to become active..."
    aws dynamodb wait table-exists --table-name "$TABLE_NAME" --region "$REGION"
    echo "Table is now active!"
else
    echo "Failed to create table. It may already exist."
    echo ""
    echo "To check if table exists:"
    echo "  aws dynamodb describe-table --table-name $TABLE_NAME --region $REGION"
    echo ""
    echo "To delete and recreate:"
    echo "  aws dynamodb delete-table --table-name $TABLE_NAME --region $REGION"
    echo "  ./create-table.sh"
fi
