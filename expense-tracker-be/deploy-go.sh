#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

REGION="${AWS_REGION:-ap-south-1}"
TABLE_NAME="expense_tracker"
FUNCTION_NAME="expense-tracker"
API_NAME="expense-tracker-api"
ROLE_NAME="expense-tracker-lambda-role"
ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)

echo "========================================"
echo "Go Lambda Deployment"
echo "========================================"
echo "Region: $REGION"
echo "Account: $ACCOUNT_ID"
echo ""

# Build the Go binary for Linux
echo "Building Go binary for Lambda..."
GOOS=linux GOARCH=amd64 go build -o bootstrap main.go

# Create zip file
zip -j bootstrap.zip bootstrap

echo ""
echo "Build complete: bootstrap.zip"
echo ""

# Create DynamoDB table if not exists
echo "Creating DynamoDB table (if not exists)..."
if ! aws dynamodb describe-table --table-name "$TABLE_NAME" --region "$REGION" 2>/dev/null; then
    aws dynamodb create-table \
        --table-name "$TABLE_NAME" \
        --attribute-definitions AttributeName=pk,AttributeType=S AttributeName=sk,AttributeType=S \
        --key-schema AttributeName=pk,KeyType=HASH AttributeName=sk,KeyType=RANGE \
        --billing-mode PAY_PER_REQUEST \
        --region "$REGION"
    echo "Waiting for table to be active..."
    aws dynamodb wait table-exists --table-name "$TABLE_NAME" --region "$REGION"
    echo "Table created successfully."
else
    echo "Table already exists."
fi

# Create IAM role
echo ""
echo "Creating IAM role (if not exists)..."
if ! aws iam get-role --role-name "$ROLE_NAME" 2>/dev/null; then
    aws iam create-role \
        --role-name "$ROLE_NAME" \
        --assume-role-policy-document '{"Version": "2012-10-17", "Statement": [{"Effect": "Allow", "Principal": {"Service": "lambda.amazonaws.com"}, "Action": "sts:AssumeRole"}]}'
    
    aws iam attach-role-policy \
        --role-name "$ROLE_NAME" \
        --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole
    
    aws iam put-role-policy \
        --role-name "$ROLE_NAME" \
        --policy-name DynamoDBAccess \
        --policy-document "{\"Version\": \"2012-10-17\", \"Statement\": [{\"Effect\": \"Allow\", \"Action\": [\"dynamodb:GetItem\", \"dynamodb:PutItem\", \"dynamodb:UpdateItem\", \"dynamodb:DeleteItem\", \"dynamodb:Query\", \"dynamodb:Scan\"], \"Resource\": \"arn:aws:dynamodb:$REGION:$ACCOUNT_ID:table/$TABLE_NAME\"}]}"
    
    echo "Waiting for role to propagate..."
    sleep 10
else
    echo "Role already exists."
fi

# Create or update Lambda function
echo ""
echo "Creating/updating Lambda function..."
if aws lambda get-function --function-name "$FUNCTION_NAME" --region "$REGION" 2>/dev/null; then
    echo "Updating existing function..."
    aws lambda update-function-code \
        --function-name "$FUNCTION_NAME" \
        --zip-file "fileb://bootstrap.zip" \
        --region "$REGION"
else
    echo "Creating new function..."
    ROLE_ARN=$(aws iam get-role --role-name "$ROLE_NAME" --query 'Role.Arn' --output text)
    aws lambda create-function \
        --function-name "$FUNCTION_NAME" \
        --runtime provided.al2023 \
        --handler bootstrap \
        --role "$ROLE_ARN" \
        --zip-file "fileb://bootstrap.zip" \
        --region "$REGION" \
        --timeout 30 \
        --memory-size 256 \
        --environment Variables="{DYNAMODB_TABLE=$TABLE_NAME,RUST_LOG=debug}"
fi

# Create API Gateway
echo ""
echo "Creating API Gateway..."
API_ID=$(aws apigatewayv2 get-apis --query "Items[?Name=='$API_NAME'].ApiId" --output text --region "$REGION")

if [ -z "$API_ID" ] || [ "$API_ID" == "None" ]; then
    API_ID=$(aws apigatewayv2 create-api \
        --name "$API_NAME" \
        --protocol-type HTTP \
        --target "arn:aws:lambda:$REGION:$ACCOUNT_ID:function:$FUNCTION_NAME" \
        --cors-configuration AllowOrigins='["*"]',AllowMethods='["*"]',AllowHeaders='["*"]' \
        --query ApiId --output text \
        --region "$REGION")
    
    aws lambda add-permission \
        --function-name "$FUNCTION_NAME" \
        --statement-id apigateway-invoke \
        --action lambda:InvokeFunction \
        --principal apigateway.amazonaws.com \
        --source-arn "arn:aws:execute-api:$REGION:$ACCOUNT_ID:$API_ID/*" \
        --region "$REGION" 2>/dev/null || true
    
    aws apigatewayv2 create-stage \
        --api-id "$API_ID" \
        --stage-name 'default' \
        --auto-deploy \
        --region "$REGION"
else
    echo "API already exists."
fi

API_URL="https://${API_ID}.execute-api.${REGION}.amazonaws.com/default"

# Cleanup
echo ""
echo "Cleaning up build files..."
rm -f bootstrap bootstrap.zip

echo ""
echo "========================================"
echo "Deployment Complete!"
echo "========================================"
echo ""
echo "API URL: $API_URL"
echo ""
echo "Example commands:"
echo "  curl $API_URL/debug"
echo "  curl $API_URL/months/03-2026/summary"
