use crate::models::{AddExpenseRequest, Expense, MonthData, UpdateAmountRequest};
use crate::state::AppState;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use chrono::Local;
use serde_json::json;
use std::collections::HashMap;

#[get("/months/{month}/expenses")]
pub async fn get_expenses(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let month = path.into_inner();
    let data = data.data.lock().unwrap();

    if let Some(month_data) = data.get(&month) {
        HttpResponse::Ok().json(&month_data.expenses)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[post("/months/{month}/expenses")]
pub async fn add_expense(
    path: web::Path<String>,
    req: web::Json<AddExpenseRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let month = path.clone();
    println!("Adding expense: {:?} for month {}", req, month);

    let month = path.into_inner();
    let mut data = state.data.lock().unwrap();
    println!("Current data state: {:?}", *data);

    let month_data = data.entry(month.clone()).or_insert(MonthData {
        expenses: HashMap::new(),
        income: HashMap::new(),
    });

    let expense = Expense {
        amount: req.amount,
        paid: false,
        date_paid: None,
    };

    println!("Inserting expense: {:?} for month {}", expense, month);
    month_data
        .expenses
        .insert(req.name.clone(), expense.clone());
    drop(data);

    println!("Saving data...");
    if let Err(e) = state.save_data() {
        println!("Error saving data: {}", e);
        return HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }));
    }
    println!("Data saved successfully");

    HttpResponse::Ok().json(json!({
        "message": "Expense added successfully",
        "expense": {
            "name": req.name,
            "data": expense
        }
    }))
}

#[put("/months/{month}/expenses/{name}/paid")]
pub async fn mark_expense_paid(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();
    println!("Marking expense as paid: {} for month {}", name, month);

    let expense_data = {
        let mut data = state.data.lock().unwrap();
        if let Some(month_data) = data.get_mut(&month) {
            if let Some(expense) = month_data.expenses.get_mut(&name) {
                expense.paid = true;
                expense.date_paid = Some(Local::now().to_string());
                Some(expense.clone())
            } else {
                return HttpResponse::NotFound().json(json!({
                    "error": format!("Expense '{}' not found in month {}", name, month)
                }));
            }
        } else {
            return HttpResponse::NotFound().json(json!({
                "error": format!("Month '{}' not found", month)
            }));
        }
    };

    if let Some(expense) = expense_data {
        if let Err(e) = state.save_data() {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to save: {}", e)
            }));
        }
        HttpResponse::Ok().json(json!({
            "message": "Expense marked as paid",
            "expense": {
                "name": name,
                "data": expense
            }
        }))
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[put("/months/{month}/expenses/{name}/unpaid")]
pub async fn mark_expense_unpaid(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();
    println!("Marking expense as unpaid: {} for month {}", name, month);

    let expense_data = {
        let mut data = state.data.lock().unwrap();
        if let Some(month_data) = data.get_mut(&month) {
            if let Some(expense) = month_data.expenses.get_mut(&name) {
                expense.paid = false;
                expense.date_paid = None;
                Some(expense.clone())
            } else {
                return HttpResponse::NotFound().json(json!({
                    "error": format!("Expense '{}' not found in month {}", name, month)
                }));
            }
        } else {
            return HttpResponse::NotFound().json(json!({
                "error": format!("Month '{}' not found", month)
            }));
        }
    };

    if let Some(expense) = expense_data {
        if let Err(e) = state.save_data() {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to save: {}", e)
            }));
        }
        HttpResponse::Ok().json(json!({
            "message": "Expense marked as unpaid",
            "expense": {
                "name": name,
                "data": expense
            }
        }))
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[put("/months/{month}/expenses/{name}/amount")]
pub async fn update_expense_amount(
    path: web::Path<(String, String)>,
    req: web::Json<UpdateAmountRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();
    println!("Updating expense amount for {}/{}", month, name);

    let expense_data = {
        let mut data = state.data.lock().unwrap();
        if let Some(month_data) = data.get_mut(&month) {
            if let Some(expense) = month_data.expenses.get_mut(&name) {
                println!(
                    "Found expense, updating amount from {} to {}",
                    expense.amount, req.amount
                );
                expense.amount = req.amount;
                Some(expense.clone())
            } else {
                let error = format!("Expense '{}' not found in month {}", name, month);
                return HttpResponse::NotFound().json(json!({ "error": error }));
            }
        } else {
            let error = format!("Month '{}' not found", month);
            return HttpResponse::NotFound().json(json!({ "error": error }));
        }
    };

    if let Some(expense) = expense_data {
        if let Err(e) = state.save_data() {
            return HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }));
        }
        HttpResponse::Ok().json(expense)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[delete("/months/{month}/expenses/{name}")]
pub async fn delete_expense(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();
    println!("Deleting expense: {} for month {}", name, month);

    let mut data = state.data.lock().unwrap();

    if let Some(month_data) = data.get_mut(&month) {
        if let Some(expense) = month_data.expenses.remove(&name) {
            drop(data);
            if let Err(e) = state.save_data() {
                return HttpResponse::InternalServerError().json(json!({
                    "error": format!("Failed to save after deletion: {}", e)
                }));
            }
            return HttpResponse::Ok().json(json!({
                "message": "Expense deleted successfully",
                "deleted": {
                    "name": name,
                    "data": expense
                }
            }));
        } else {
            return HttpResponse::NotFound().json(json!({
                "error": format!("Expense '{}' not found in month {}", name, month)
            }));
        }
    }

    HttpResponse::NotFound().json(json!({
        "error": format!("Month '{}' not found", month)
    }))
}
