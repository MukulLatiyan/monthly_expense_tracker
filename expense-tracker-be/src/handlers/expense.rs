use crate::models::{AddExpenseRequest, ApiResponse, ExpenseListResponse, UpdateAmountRequest};
use crate::state::{AppState, RepositoryError};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

fn map_error_to_response(err: RepositoryError) -> HttpResponse {
    match err {
        RepositoryError::NotFound(msg) => HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Expense not found",
            msg,
        )),
        RepositoryError::Validation(msg) => {
            HttpResponse::BadRequest().json(ApiResponse::<()>::error("Validation error", msg))
        }
        RepositoryError::DynamoDb(msg) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Database error", msg))
        }
        RepositoryError::Serialization(msg) => HttpResponse::InternalServerError().json(
            ApiResponse::<()>::error("Data serialization error", msg),
        ),
    }
}

/// Get all expenses for a month (returns empty list if month has no expenses)
#[get("/months/{month}/expenses")]
pub async fn get_expenses(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let month = path.into_inner();

    match state.get_expenses(&month).await {
        Ok(expenses) => {
            let response = ExpenseListResponse {
                month: month.clone(),
                count: expenses.len(),
                expenses,
            };

            if response.count == 0 {
                HttpResponse::Ok().json(ApiResponse::success(
                    response,
                    format!("No expenses found for month {}. The month may not have any expenses yet.", month),
                ))
            } else {
                HttpResponse::Ok()
                    .json(ApiResponse::success(response, "Expenses retrieved successfully"))
            }
        }
        Err(e) => map_error_to_response(e),
    }
}

/// Add a new expense to a month
#[post("/months/{month}/expenses")]
pub async fn add_expense(
    path: web::Path<String>,
    req: web::Json<AddExpenseRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let month = path.into_inner();

    // Validate request
    if req.name.trim().is_empty() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Validation error",
            "Expense name cannot be empty",
        ));
    }

    if req.amount < 0.0 {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Validation error",
            "Expense amount cannot be negative",
        ));
    }

    match state.add_expense(&month, &req.name, req.amount).await {
        Ok(expense) => HttpResponse::Created().json(ApiResponse::success(
            expense,
            format!("Expense '{}' added successfully to month {}", req.name, month),
        )),
        Err(e) => map_error_to_response(e),
    }
}

/// Mark an expense as paid
#[put("/months/{month}/expenses/{name}/paid")]
pub async fn mark_expense_paid(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();

    match state.mark_expense_paid(&month, &name).await {
        Ok(expense) => HttpResponse::Ok().json(ApiResponse::success(
            expense,
            format!("Expense '{}' marked as paid for month {}", name, month),
        )),
        Err(e) => map_error_to_response(e),
    }
}

/// Mark an expense as unpaid
#[put("/months/{month}/expenses/{name}/unpaid")]
pub async fn mark_expense_unpaid(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();

    match state.mark_expense_unpaid(&month, &name).await {
        Ok(expense) => HttpResponse::Ok().json(ApiResponse::success(
            expense,
            format!("Expense '{}' marked as unpaid for month {}", name, month),
        )),
        Err(e) => map_error_to_response(e),
    }
}

/// Update an expense amount
#[put("/months/{month}/expenses/{name}/amount")]
pub async fn update_expense_amount(
    path: web::Path<(String, String)>,
    req: web::Json<UpdateAmountRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();

    // Validate request
    if req.amount < 0.0 {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Validation error",
            "Expense amount cannot be negative",
        ));
    }

    match state.update_expense_amount(&month, &name, req.amount).await {
        Ok(expense) => HttpResponse::Ok().json(ApiResponse::success(
            expense,
            format!(
                "Expense '{}' amount updated to {} for month {}",
                name, req.amount, month
            ),
        )),
        Err(e) => map_error_to_response(e),
    }
}

/// Delete an expense
#[delete("/months/{month}/expenses/{name}")]
pub async fn delete_expense(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();

    match state.delete_expense(&month, &name).await {
        Ok(deleted_expense) => HttpResponse::Ok().json(ApiResponse::success(
            deleted_expense,
            format!("Expense '{}' deleted successfully from month {}", name, month),
        )),
        Err(e) => map_error_to_response(e),
    }
}
