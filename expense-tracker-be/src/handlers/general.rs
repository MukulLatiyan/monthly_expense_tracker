use crate::models::{ApiResponse, SummaryResponse};
use crate::state::{AppState, RepositoryError};
use actix_web::{get, web, HttpResponse, Responder};

fn map_error_to_response(err: RepositoryError) -> HttpResponse {
    match err {
        RepositoryError::NotFound(msg) => HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Resource not found",
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

/// Debug endpoint - returns usage information
#[get("/debug")]
pub async fn debug_data() -> impl Responder {
    // Debug endpoint returns usage info - for detailed data use specific month endpoints
    HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({"message": "Use /months/{month}/summary for month data"}),
        "Debug endpoint - use month-specific endpoints for data"
    ))
}

/// Get summary for a specific month
#[get("/months/{month}/summary")]
pub async fn get_summary(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let month = path.into_inner();

    match state.get_month_summary(&month).await {
        Ok(month_data) => {
            let total_expenses: f64 = month_data.expenses.iter().map(|e| e.data.amount).sum();
            let total_income: f64 = month_data.income.iter().map(|i| i.data.amount).sum();
            let received_income: f64 = month_data
                .income
                .iter()
                .filter(|i| i.data.received)
                .map(|i| i.data.amount)
                .sum();
            let pending_income = total_income - received_income;
            let balance = total_income - total_expenses;

            let summary = SummaryResponse {
                month: month.clone(),
                total_income,
                received_income,
                pending_income,
                total_expenses,
                balance,
                spending_ratio: if total_income > 0.0 {
                    (total_expenses / total_income) * 100.0
                } else {
                    0.0
                },
                expense_count: month_data.expenses.len(),
                income_count: month_data.income.len(),
            };

            HttpResponse::Ok()
                .json(ApiResponse::success(summary, "Monthly summary retrieved successfully"))
        }
        Err(e) => map_error_to_response(e),
    }
}
