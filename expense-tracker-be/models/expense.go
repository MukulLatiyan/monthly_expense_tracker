package models

import "time"

type Expense struct {
	Month     string  `json:"month" dynamodbav:"month"`
	Name      string  `json:"name" dynamodbav:"name"`
	Amount    float64 `json:"amount" dynamodbav:"amount"`
	Paid      bool    `json:"paid" dynamodbav:"paid"`
	DatePaid  *string `json:"date_paid,omitempty" dynamodbav:"date_paid,omitempty"`
	CreatedAt string  `json:"created_at" dynamodbav:"created_at"`
}

type Income struct {
	Month         string  `json:"month" dynamodbav:"month"`
	Name          string  `json:"name" dynamodbav:"name"`
	Amount        float64 `json:"amount" dynamodbav:"amount"`
	Received      bool    `json:"received" dynamodbav:"received"`
	DateReceived  *string `json:"date_received,omitempty" dynamodbav:"date_received,omitempty"`
	CreatedAt     string  `json:"created_at" dynamodbav:"created_at"`
}

type AddExpenseRequest struct {
	Name   string  `json:"name" binding:"required"`
	Amount float64 `json:"amount" binding:"required,min=0"`
}

type AddIncomeRequest struct {
	Name   string  `json:"name" binding:"required"`
	Amount float64 `json:"amount" binding:"required,min=0"`
}

type UpdateAmountRequest struct {
	Amount float64 `json:"amount" binding:"required,min=0"`
}

type ApiResponse struct {
	Success bool        `json:"success"`
	Data    interface{} `json:"data"`
	Message string      `json:"message"`
	Error   string      `json:"error,omitempty"`
}

type SummaryResponse struct {
	Month          string  `json:"month"`
	TotalIncome    float64 `json:"total_income"`
	ReceivedIncome float64 `json:"received_income"`
	PendingIncome  float64 `json:"pending_income"`
	TotalExpenses  float64 `json:"total_expenses"`
	PaidExpenses   float64 `json:"paid_expenses"`
	Balance        float64 `json:"balance"`
	ActualBalance  float64 `json:"actual_balance"`
	ExpenseCount   int     `json:"expense_count"`
	IncomeCount    int     `json:"income_count"`
}

func NewSuccessResponse(data interface{}, message string) ApiResponse {
	return ApiResponse{
		Success: true,
		Data:    data,
		Message: message,
	}
}

func NewErrorResponse(message, err string) ApiResponse {
	return ApiResponse{
		Success: false,
		Data:    nil,
		Message: message,
		Error:   err,
	}
}

func GetCurrentTimestamp() string {
	return time.Now().Format(time.RFC3339)
}
