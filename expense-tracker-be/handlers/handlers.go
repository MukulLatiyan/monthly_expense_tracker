package handlers

import (
	"context"
	"fmt"
	"net/url"
	"strconv"

	"expense-tracker/db"
	"expense-tracker/models"

	"github.com/gofiber/fiber/v2"
)

// decodeParam URL-decodes a path parameter
func decodeParam(s string) string {
	decoded, err := url.PathUnescape(s)
	if err != nil {
		return s
	}
	return decoded
}

type Handler struct {
	Repo *db.DynamoDBRepository
}

func NewHandler(repo *db.DynamoDBRepository) *Handler {
	return &Handler{Repo: repo}
}

func (h *Handler) SetupRoutes(r fiber.Router) {
	r.Get("/", h.RootHandler)
	r.Get("/debug", h.DebugData)
	r.Get("/months/:month/summary", h.GetSummary)

	r.Get("/months/:month/expenses", h.GetExpenses)
	r.Post("/months/:month/expenses", h.AddExpense)
	r.Put("/months/:month/expenses/:name/paid", h.MarkExpensePaid)
	r.Put("/months/:month/expenses/:name/unpaid", h.MarkExpenseUnpaid)
	r.Put("/months/:month/expenses/:name/amount", h.UpdateExpenseAmount)
	r.Delete("/months/:month/expenses/:name", h.DeleteExpense)

	r.Get("/months/:month/income", h.GetIncome)
	r.Post("/months/:month/income", h.AddIncome)
	r.Put("/months/:month/income/:name/received", h.MarkIncomeReceived)
	r.Put("/months/:month/income/:name/unreceived", h.MarkIncomeUnreceived)
	r.Put("/months/:month/income/:name/amount", h.UpdateIncomeAmount)
	r.Delete("/months/:month/income/:name", h.DeleteIncome)
}

func (h *Handler) RootHandler(c *fiber.Ctx) error {
	return c.JSON(models.ApiResponse{
		Success: true,
		Data: map[string]string{
			"message": "Expense Tracker API",
			"version": "1.0",
			"status": "running",
		},
		Message: "API is running",
	})
}

func (h *Handler) DebugData(c *fiber.Ctx) error {
	return c.JSON(models.ApiResponse{
		Success: true,
		Data: map[string]string{
			"table": h.Repo.TableName,
			"region": "ap-south-1",
		},
		Message: "Debug info",
	})
}

func (h *Handler) GetSummary(c *fiber.Ctx) error {
	month := decodeParam(c.Params("month"))
	ctx := context.Background()

	expenses, err := h.Repo.GetExpenses(ctx, month)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Failed to get expenses", err.Error()))
	}

	income, err := h.Repo.GetIncome(ctx, month)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Failed to get income", err.Error()))
	}

	var totalExpenses, paidExpenses float64
	for _, e := range expenses {
		totalExpenses += e.Amount
		if e.Paid {
			paidExpenses += e.Amount
		}
	}

	var totalIncome, receivedIncome float64
	for _, i := range income {
		totalIncome += i.Amount
		if i.Received {
			receivedIncome += i.Amount
		}
	}

	summary := models.SummaryResponse{
		Month:          month,
		TotalIncome:    totalIncome,
		ReceivedIncome: receivedIncome,
		PendingIncome:  totalIncome - receivedIncome,
		TotalExpenses:  totalExpenses,
		PaidExpenses:   paidExpenses,
		Balance:        totalIncome - totalExpenses,
		ActualBalance:  receivedIncome - paidExpenses,
		ExpenseCount:   len(expenses),
		IncomeCount:    len(income),
	}

	return c.JSON(models.NewSuccessResponse(summary, "Summary retrieved"))
}

func (h *Handler) GetExpenses(c *fiber.Ctx) error {
	month := decodeParam(c.Params("month"))
	ctx := context.Background()

	expenses, err := h.Repo.GetExpenses(ctx, month)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Failed to get expenses", err.Error()))
	}

	return c.JSON(models.NewSuccessResponse(expenses, "Expenses retrieved"))
}

func (h *Handler) AddExpense(c *fiber.Ctx) error {
	month := decodeParam(c.Params("month"))
	var req models.AddExpenseRequest

	if err := c.BodyParser(&req); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(models.NewErrorResponse("Invalid request", err.Error()))
	}

	if req.Name == "" {
		return c.Status(fiber.StatusBadRequest).JSON(models.NewErrorResponse("Validation error", "Name is required"))
	}

	ctx := context.Background()

	existing, err := h.Repo.GetExpense(ctx, month, req.Name)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Database error", err.Error()))
	}
	if existing != nil {
		return c.Status(fiber.StatusConflict).JSON(models.NewErrorResponse("Validation error", fmt.Sprintf("Expense '%s' already exists", req.Name)))
	}

	expense := models.Expense{
		Month:     month,
		Name:      req.Name,
		Amount:    req.Amount,
		Paid:      false,
		CreatedAt: models.GetCurrentTimestamp(),
	}

	if err := h.Repo.AddExpense(ctx, expense); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Failed to add expense", err.Error()))
	}

	return c.Status(fiber.StatusCreated).JSON(models.NewSuccessResponse(expense, "Expense added"))
}

func (h *Handler) MarkExpensePaid(c *fiber.Ctx) error {
	return h.updateExpenseStatus(c, true)
}

func (h *Handler) MarkExpenseUnpaid(c *fiber.Ctx) error {
	return h.updateExpenseStatus(c, false)
}

func (h *Handler) updateExpenseStatus(c *fiber.Ctx, paid bool) error {
	month := decodeParam(c.Params("month"))
	name := decodeParam(c.Params("name"))
	ctx := context.Background()

	expense, err := h.Repo.GetExpense(ctx, month, name)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Database error", err.Error()))
	}
	if expense == nil {
		return c.Status(fiber.StatusNotFound).JSON(models.NewErrorResponse("Not found", fmt.Sprintf("Expense '%s' not found", name)))
	}

	expense.Paid = paid
	now := models.GetCurrentTimestamp()
	if paid {
		expense.DatePaid = &now
	} else {
		expense.DatePaid = nil
	}

	if err := h.Repo.UpdateExpense(ctx, *expense); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Failed to update expense", err.Error()))
	}

	status := "unpaid"
	if paid {
		status = "paid"
	}
	return c.JSON(models.NewSuccessResponse(expense, fmt.Sprintf("Expense marked as %s", status)))
}

func (h *Handler) UpdateExpenseAmount(c *fiber.Ctx) error {
	month := decodeParam(c.Params("month"))
	name := decodeParam(c.Params("name"))
	var req models.UpdateAmountRequest

	if err := c.BodyParser(&req); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(models.NewErrorResponse("Invalid request", err.Error()))
	}

	ctx := context.Background()

	expense, err := h.Repo.GetExpense(ctx, month, name)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Database error", err.Error()))
	}
	if expense == nil {
		return c.Status(fiber.StatusNotFound).JSON(models.NewErrorResponse("Not found", fmt.Sprintf("Expense '%s' not found", name)))
	}

	expense.Amount = req.Amount

	if err := h.Repo.UpdateExpense(ctx, *expense); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Failed to update expense", err.Error()))
	}

	return c.JSON(models.NewSuccessResponse(expense, "Expense amount updated"))
}

func (h *Handler) DeleteExpense(c *fiber.Ctx) error {
	month := decodeParam(c.Params("month"))
	name := decodeParam(c.Params("name"))
	ctx := context.Background()

	expense, err := h.Repo.GetExpense(ctx, month, name)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Database error", err.Error()))
	}
	if expense == nil {
		return c.Status(fiber.StatusNotFound).JSON(models.NewErrorResponse("Not found", fmt.Sprintf("Expense '%s' not found", name)))
	}

	if err := h.Repo.DeleteExpense(ctx, month, name); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Failed to delete expense", err.Error()))
	}

	return c.JSON(models.NewSuccessResponse(nil, "Expense deleted"))
}

func (h *Handler) GetIncome(c *fiber.Ctx) error {
	month := decodeParam(c.Params("month"))
	ctx := context.Background()

	income, err := h.Repo.GetIncome(ctx, month)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Failed to get income", err.Error()))
	}

	return c.JSON(models.NewSuccessResponse(income, "Income retrieved"))
}

func (h *Handler) AddIncome(c *fiber.Ctx) error {
	month := decodeParam(c.Params("month"))
	var req models.AddIncomeRequest

	if err := c.BodyParser(&req); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(models.NewErrorResponse("Invalid request", err.Error()))
	}

	if req.Name == "" {
		return c.Status(fiber.StatusBadRequest).JSON(models.NewErrorResponse("Validation error", "Name is required"))
	}

	ctx := context.Background()

	existing, err := h.Repo.GetIncomeItem(ctx, month, req.Name)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Database error", err.Error()))
	}
	if existing != nil {
		return c.Status(fiber.StatusConflict).JSON(models.NewErrorResponse("Validation error", fmt.Sprintf("Income '%s' already exists", req.Name)))
	}

	income := models.Income{
		Month:     month,
		Name:      req.Name,
		Amount:    req.Amount,
		Received:  false,
		CreatedAt: models.GetCurrentTimestamp(),
	}

	if err := h.Repo.AddIncome(ctx, income); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Failed to add income", err.Error()))
	}

	return c.Status(fiber.StatusCreated).JSON(models.NewSuccessResponse(income, "Income added"))
}

func (h *Handler) MarkIncomeReceived(c *fiber.Ctx) error {
	return h.updateIncomeStatus(c, true)
}

func (h *Handler) MarkIncomeUnreceived(c *fiber.Ctx) error {
	return h.updateIncomeStatus(c, false)
}

func (h *Handler) updateIncomeStatus(c *fiber.Ctx, received bool) error {
	month := decodeParam(c.Params("month"))
	name := decodeParam(c.Params("name"))
	ctx := context.Background()

	income, err := h.Repo.GetIncomeItem(ctx, month, name)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Database error", err.Error()))
	}
	if income == nil {
		return c.Status(fiber.StatusNotFound).JSON(models.NewErrorResponse("Not found", fmt.Sprintf("Income '%s' not found", name)))
	}

	income.Received = received
	now := models.GetCurrentTimestamp()
	if received {
		income.DateReceived = &now
	} else {
		income.DateReceived = nil
	}

	if err := h.Repo.UpdateIncome(ctx, *income); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Failed to update income", err.Error()))
	}

	status := "not received"
	if received {
		status = "received"
	}
	return c.JSON(models.NewSuccessResponse(income, fmt.Sprintf("Income marked as %s", status)))
}

func (h *Handler) UpdateIncomeAmount(c *fiber.Ctx) error {
	month := decodeParam(c.Params("month"))
	name := decodeParam(c.Params("name"))
	var req models.UpdateAmountRequest

	if err := c.BodyParser(&req); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(models.NewErrorResponse("Invalid request", err.Error()))
	}

	ctx := context.Background()

	income, err := h.Repo.GetIncomeItem(ctx, month, name)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Database error", err.Error()))
	}
	if income == nil {
		return c.Status(fiber.StatusNotFound).JSON(models.NewErrorResponse("Not found", fmt.Sprintf("Income '%s' not found", name)))
	}

	income.Amount = req.Amount

	if err := h.Repo.UpdateIncome(ctx, *income); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Failed to update income", err.Error()))
	}

	return c.JSON(models.NewSuccessResponse(income, "Income amount updated"))
}

func (h *Handler) DeleteIncome(c *fiber.Ctx) error {
	month := decodeParam(c.Params("month"))
	name := decodeParam(c.Params("name"))
	ctx := context.Background()

	income, err := h.Repo.GetIncomeItem(ctx, month, name)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Database error", err.Error()))
	}
	if income == nil {
		return c.Status(fiber.StatusNotFound).JSON(models.NewErrorResponse("Not found", fmt.Sprintf("Income '%s' not found", name)))
	}

	if err := h.Repo.DeleteIncome(ctx, month, name); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(models.NewErrorResponse("Failed to delete income", err.Error()))
	}

	return c.JSON(models.NewSuccessResponse(nil, "Income deleted"))
}

func parseFloat(s string) (float64, error) {
	return strconv.ParseFloat(s, 64)
}
