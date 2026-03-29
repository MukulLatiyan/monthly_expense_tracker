#!/bin/bash
set -e

REGION="${AWS_REGION:-ap-south-1}"
TABLE_NAME="expense_tracker"
FUNCTION_NAME="expense-tracker"
API_NAME="expense-tracker-api"
ROLE_NAME="expense-tracker-lambda-role"

echo "========================================"
echo "Cleaning up AWS Resources"
echo "========================================"
echo "Region: $REGION"
echo ""

# Delete API Gateway
echo "Deleting API Gateway..."
API_ID=$(aws apigatewayv2 get-apis --query "Items[?Name=='$API_NAME'].ApiId" --output text --region "$REGION")
if [ -n "$API_ID" ] && [ "$API_ID" != "None" ]; then
    aws apigatewayv2 delete-api --api-id "$API_ID" --region "$REGION" || true
    echo "API Gateway deleted."
else
    echo "API Gateway not found."
fi

# Delete Lambda function
echo ""
echo "Deleting Lambda function..."
if aws lambda get-function --function-name "$FUNCTION_NAME" --region "$REGION" 2>/dev/null; then
    aws lambda delete-function --function-name "$FUNCTION_NAME" --region "$REGION" || true
    echo "Lambda function deleted."
else
    echo "Lambda function not found."
fi

# Detach policies and delete IAM role
echo ""
echo "Cleaning up IAM role..."
if aws iam get-role --role-name "$ROLE_NAME" 2>/dev/null; then
    # Detach managed policies
    aws iam detach-role-policy \
        --role-name "$ROLE_NAME" \
        --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole \
        --region "$REGION" 2>/dev/null || true
    
    # Delete inline policies
    aws iam delete-role-policy \
        --role-name "$ROLE_NAME" \
        --policy-name DynamoDBAccess \
        --region "$REGION" 2>/dev/null || true
    
    # Delete the role
    aws iam delete-role --role-name "$ROLE_NAME" --region "$REGION" || true
    echo "IAM role deleted."
else
    echo "IAM role not found."
fi

# Delete DynamoDB table (optional - commented out for safety)
# echo ""
# echo "Deleting DynamoDB table..."
# if aws dynamodb describe-table --table-name "$TABLE_NAME" --region "$REGION" 2>/dev/null; then
#     aws dynamodb delete-table --table-name "$TABLE_NAME" --region "$REGION" || true
#     echo "DynamoDB table deleted."
# else
#     echo "DynamoDB table not found."
# fi

echo ""
echo "========================================"
echo "Cleanup Complete!"
echo "========================================"
echo ""
echo "Note: DynamoDB table '$TABLE_NAME' was preserved."
echo "To delete it, run: aws dynamodb delete-table --table-name $TABLE_NAME --region $REGION"
