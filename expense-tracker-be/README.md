# Expense Tracker Backend

A Rust-based REST API for tracking monthly expenses and income, backed by AWS DynamoDB.

## Features

- **RESTful API** with Actix Web
- **AWS DynamoDB** for persistent storage
- **Proper error handling** with detailed JSON responses
- **CORS enabled** for frontend integration
- **Request logging** for debugging

## Prerequisites

- Rust (latest stable version)
- AWS CLI configured with credentials
- Docker (for local DynamoDB development)

## Setup

### 1. AWS Credentials

Configure AWS CLI with your credentials:

```bash
aws configure
# Enter your AWS Access Key ID, Secret Access Key, and region (ap-south-1)
```

### 2. Create DynamoDB Table

The application requires a DynamoDB table to store data.

**Option A: Use AWS DynamoDB (Production)**

Run the helper script to create the table:

```bash
./create-table.sh
```

Or manually create via AWS CLI:

```bash
aws dynamodb create-table \
    --table-name expense_tracker \
    --attribute-definitions \
        AttributeName=pk,AttributeType=S \
        AttributeName=sk,AttributeType=S \
    --key-schema \
        AttributeName=pk,KeyType=HASH \
        AttributeName=sk,KeyType=RANGE \
    --billing-mode PAY_PER_REQUEST \
    --region ap-south-1
```

**Option B: Use Local DynamoDB (Development)**

Run the setup script which starts a Docker container and creates the table:

```bash
./setup-local.sh
```

Then run the server with:

```bash
export DYNAMODB_LOCAL=true
export DYNAMODB_ENDPOINT=http://localhost:8000
cargo run
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DYNAMODB_TABLE` | `expense_tracker` | DynamoDB table name |
| `AWS_REGION` | `ap-south-1` | AWS region |
| `DYNAMODB_LOCAL` | `false` | Use local DynamoDB |
| `DYNAMODB_ENDPOINT` | `http://localhost:8000` | Local DynamoDB endpoint |
| `SERVER_HOST` | `127.0.0.1` | Server bind address |
| `SERVER_PORT` | `8080` | Server port |
| `RUST_LOG` | `debug` | Log level |

## Running the Server

```bash
# Development with local DynamoDB
export DYNAMODB_LOCAL=true
export DYNAMODB_ENDPOINT=http://localhost:8000
cargo run

# Production with AWS DynamoDB
cargo run
```

## API Endpoints

### General

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/debug` | Debug endpoint |
| GET | `/months/{month}/summary` | Get monthly summary |

### Expenses

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/months/{month}/expenses` | List all expenses for month |
| POST | `/months/{month}/expenses` | Add new expense |
| PUT | `/months/{month}/expenses/{name}/paid` | Mark expense as paid |
| PUT | `/months/{month}/expenses/{name}/unpaid` | Mark expense as unpaid |
| PUT | `/months/{month}/expenses/{name}/amount` | Update expense amount |
| DELETE | `/months/{month}/expenses/{name}` | Delete expense |

### Income

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/months/{month}/income` | List all income for month |
| POST | `/months/{month}/income` | Add new income |
| PUT | `/months/{month}/income/{name}/received` | Mark income as received |
| PUT | `/months/{month}/income/{name}/unreceived` | Mark income as unreceived |
| PUT | `/months/{month}/income/{name}/amount` | Update income amount |
| DELETE | `/months/{month}/income/{name}` | Delete income |

## API Response Format

All API responses follow a consistent JSON format:

### Success Response

```json
{
  "success": true,
  "data": { ... },
  "message": "Operation completed successfully"
}
```

### Error Response

```json
{
  "success": false,
  "data": null,
  "message": "Error category",
  "error": "Detailed error message"
}
```

## Example Usage

### Add an Expense

```bash
curl -X POST http://localhost:8080/months/03-2026/expenses \
  -H "Content-Type: application/json" \
  -d '{"name": "Rent", "amount": 20000}'
```

Response:
```json
{
  "success": true,
  "data": {
    "month": "03-2026",
    "name": "Rent",
    "data": {
      "amount": 20000,
      "paid": false,
      "date_paid": null
    }
  },
  "message": "Expense 'Rent' added successfully to month 03-2026"
}
```

### Get Monthly Summary

```bash
curl http://localhost:8080/months/03-2026/summary
```

Response:
```json
{
  "success": true,
  "data": {
    "month": "03-2026",
    "total_income": 50000,
    "received_income": 50000,
    "pending_income": 0,
    "total_expenses": 25000,
    "balance": 25000,
    "spending_ratio": 50,
    "expense_count": 2,
    "income_count": 1
  },
  "message": "Monthly summary retrieved successfully"
}
```

### Get Empty Month (Returns 200, not 404)

```bash
curl http://localhost:8080/months/99-2099/expenses
```

Response:
```json
{
  "success": true,
  "data": {
    "month": "99-2099",
    "expenses": [],
    "count": 0
  },
  "message": "No expenses found for month 99-2099. The month may not have any expenses yet."
}
```

## DynamoDB Table Schema

**Table Name:** `expense_tracker`

**Partition Key (PK):** `MONTH#{month}` (e.g., `MONTH#03-2026`)

**Sort Key (SK):**
- Expenses: `EXPENSE#{name}` (e.g., `EXPENSE#Rent`)
- Income: `INCOME#{name}` (e.g., `INCOME#Salary`)

**Attributes:**
- `month` - Month identifier
- `name` - Expense/Income name
- `type` - `"expense"` or `"income"`
- `amount` - Numeric amount
- `paid`/`received` - Boolean status
- `date_paid`/`date_received` - Optional timestamp
- `data` - Serialized JSON blob

## Common Errors

### "ResourceNotFoundException: Cannot do operations on a non-existent table"

The DynamoDB table doesn't exist. Run `./create-table.sh` to create it.

### "UnrecognizedClientException: The security token included in the request is invalid"

AWS credentials are not configured or expired. Run `aws configure` to update them.

## Development

### View Table Data (Local DynamoDB)

```bash
aws dynamodb scan \
    --table-name expense_tracker \
    --endpoint-url http://localhost:8000 \
    --region ap-south-1
```

### Delete Table (Local)

```bash
aws dynamodb delete-table \
    --table-name expense_tracker \
    --endpoint-url http://localhost:8000 \
    --region ap-south-1
```

### Stop Local DynamoDB

```bash
docker stop dynamodb-local
```

## License

MIT
