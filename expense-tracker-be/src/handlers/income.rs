use crate::models::{AddIncomeRequest, ApiResponse, IncomeListResponse, UpdateAmountRequest};
use crate::state::{AppState, RepositoryError};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

fn map_error_to_response(err: RepositoryError) -> HttpResponse {
    match err {
        RepositoryError::NotFound(msg) => {
            HttpResponse::NotFound().json(ApiResponse::<()>::error("Income not found", msg))
        }
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

/// Get all income for a month (returns empty list if month has no income)
#[get("/months/{month}/income")]
pub async fn get_income(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let month = path.into_inner();

    match state.get_income(&month).await {
        Ok(income) => {
            let response = IncomeListResponse {
                month: month.clone(),
                count: income.len(),
                income,
            };

            if response.count == 0 {
                HttpResponse::Ok().json(ApiResponse::success(
                    response,
                    format!("No income entries found for month {}. The month may not have any income yet.", month),
                ))
            } else {
                HttpResponse::Ok()
                    .json(ApiResponse::success(response, "Income retrieved successfully"))
            }
        }
        Err(e) => map_error_to_response(e),
    }
}

/// Add a new income entry to a month
#[post("/months/{month}/income")]
pub async fn add_income(
    path: web::Path<String>,
    req: web::Json<AddIncomeRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let month = path.into_inner();

    // Validate request
    if req.name.trim().is_empty() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Validation error",
            "Income name cannot be empty",
        ));
    }

    if req.amount < 0.0 {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Validation error",
            "Income amount cannot be negative",
        ));
    }

    match state.add_income(&month, &req.name, req.amount).await {
        Ok(income) => HttpResponse::Created().json(ApiResponse::success(
            income,
            format!("Income '{}' added successfully to month {}", req.name, month),
        )),
        Err(e) => map_error_to_response(e),
    }
}

/// Mark an income as received
#[put("/months/{month}/income/{name}/received")]
pub async fn mark_income_received(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();

    match state.mark_income_received(&month, &name).await {
        Ok(income) => HttpResponse::Ok().json(ApiResponse::success(
            income,
            format!("Income '{}' marked as received for month {}", name, month),
        )),
        Err(e) => map_error_to_response(e),
    }
}

/// Mark an income as unreceived
#[put("/months/{month}/income/{name}/unreceived")]
pub async fn mark_income_unreceived(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();

    match state.mark_income_unreceived(&month, &name).await {
        Ok(income) => HttpResponse::Ok().json(ApiResponse::success(
            income,
            format!("Income '{}' marked as unreceived for month {}", name, month),
        )),
        Err(e) => map_error_to_response(e),
    }
}

/// Update an income amount
#[put("/months/{month}/income/{name}/amount")]
pub async fn update_income_amount(
    path: web::Path<(String, String)>,
    req: web::Json<UpdateAmountRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();

    // Validate request
    if req.amount < 0.0 {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Validation error",
            "Income amount cannot be negative",
        ));
    }

    match state.update_income_amount(&month, &name, req.amount).await {
        Ok(income) => HttpResponse::Ok().json(ApiResponse::success(
            income,
            format!(
                "Income '{}' amount updated to {} for month {}",
                name, req.amount, month
            ),
        )),
        Err(e) => map_error_to_response(e),
    }
}

/// Delete an income entry
#[delete("/months/{month}/income/{name}")]
pub async fn delete_income(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();

    match state.delete_income(&month, &name).await {
        Ok(deleted_income) => HttpResponse::Ok().json(ApiResponse::success(
            deleted_income,
            format!("Income '{}' deleted successfully from month {}", name, month),
        )),
        Err(e) => map_error_to_response(e),
    }
}
