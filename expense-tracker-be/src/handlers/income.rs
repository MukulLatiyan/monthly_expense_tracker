use crate::models::{AddIncomeRequest, Income, MonthData, UpdateAmountRequest};
use crate::state::AppState;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use chrono::Local;
use serde_json::json;
use std::collections::HashMap;

#[get("/months/{month}/income")]
pub async fn get_income(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let month = path.into_inner();
    println!("Requested month: {}", month);
    let data = data.data.lock().unwrap();
    println!("Available data: {:?}", *data);

    if let Some(month_data) = data.get(&month) {
        println!("Found data for month");
        HttpResponse::Ok().json(&month_data.income)
    } else {
        println!("No data found for month");
        HttpResponse::NotFound().finish()
    }
}

#[post("/months/{month}/income")]
pub async fn add_income(
    path: web::Path<String>,
    req: web::Json<AddIncomeRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let month = path.clone();
    println!("Adding income: {:?} for month {}", req, month);

    let month = path.into_inner();
    let mut data = state.data.lock().unwrap();
    println!("Current data state: {:?}", *data);

    let month_data = data.entry(month.clone()).or_insert(MonthData {
        expenses: HashMap::new(),
        income: HashMap::new(),
    });

    let income = Income {
        amount: req.amount,
        received: false,
        date_received: None,
    };

    println!("Inserting income: {:?} for month {}", income, month);
    month_data.income.insert(req.name.clone(), income.clone());
    drop(data);

    println!("Saving data...");
    if let Err(e) = state.save_data() {
        println!("Error saving data: {}", e);
        return HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }));
    }
    println!("Data saved successfully");

    HttpResponse::Ok().json(json!({
        "message": "Income added successfully",
        "income": {
            "name": req.name,
            "data": income
        }
    }))
}

#[put("/months/{month}/income/{name}/received")]
pub async fn mark_income_received(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();
    println!("Marking income as received: {} for month {}", name, month);

    let income_data = {
        let mut data = state.data.lock().unwrap();
        if let Some(month_data) = data.get_mut(&month) {
            if let Some(income) = month_data.income.get_mut(&name) {
                income.received = true;
                income.date_received = Some(Local::now().to_string());
                Some(income.clone())
            } else {
                return HttpResponse::NotFound().json(json!({
                    "error": format!("Income '{}' not found in month {}", name, month)
                }));
            }
        } else {
            return HttpResponse::NotFound().json(json!({
                "error": format!("Month '{}' not found", month)
            }));
        }
    };

    if let Some(income) = income_data {
        if let Err(e) = state.save_data() {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to save: {}", e)
            }));
        }
        HttpResponse::Ok().json(json!({
            "message": "Income marked as received",
            "income": {
                "name": name,
                "data": income
            }
        }))
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[put("/months/{month}/income/{name}/unreceived")]
pub async fn mark_income_unreceived(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();
    println!("Marking income as unreceived: {} for month {}", name, month);

    let income_data = {
        let mut data = state.data.lock().unwrap();
        if let Some(month_data) = data.get_mut(&month) {
            if let Some(income) = month_data.income.get_mut(&name) {
                income.received = false;
                income.date_received = None;
                Some(income.clone())
            } else {
                return HttpResponse::NotFound().json(json!({
                    "error": format!("Income '{}' not found in month {}", name, month)
                }));
            }
        } else {
            return HttpResponse::NotFound().json(json!({
                "error": format!("Month '{}' not found", month)
            }));
        }
    };

    if let Some(income) = income_data {
        if let Err(e) = state.save_data() {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to save: {}", e)
            }));
        }
        HttpResponse::Ok().json(json!({
            "message": "Income marked as unreceived",
            "income": {
                "name": name,
                "data": income
            }
        }))
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[put("/months/{month}/income/{name}/amount")]
pub async fn update_income_amount(
    path: web::Path<(String, String)>,
    req: web::Json<UpdateAmountRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();
    println!("Updating income amount for {}/{}", month, name);

    let income_data = {
        let mut data = state.data.lock().unwrap();
        if let Some(month_data) = data.get_mut(&month) {
            if let Some(income) = month_data.income.get_mut(&name) {
                println!(
                    "Found income, updating amount from {} to {}",
                    income.amount, req.amount
                );
                income.amount = req.amount;
                Some(income.clone())
            } else {
                let error = format!("Income '{}' not found in month {}", name, month);
                return HttpResponse::NotFound().json(json!({ "error": error }));
            }
        } else {
            let error = format!("Month '{}' not found", month);
            return HttpResponse::NotFound().json(json!({ "error": error }));
        }
    };

    if let Some(income) = income_data {
        if let Err(e) = state.save_data() {
            return HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }));
        }
        HttpResponse::Ok().json(income)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[delete("/months/{month}/income/{name}")]
pub async fn delete_income(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> impl Responder {
    let (month, name) = path.into_inner();
    println!("Deleting income: {} for month {}", name, month);

    let mut data = state.data.lock().unwrap();

    if let Some(month_data) = data.get_mut(&month) {
        if let Some(income) = month_data.income.remove(&name) {
            drop(data);
            if let Err(e) = state.save_data() {
                return HttpResponse::InternalServerError().json(json!({
                    "error": format!("Failed to save after deletion: {}", e)
                }));
            }
            return HttpResponse::Ok().json(json!({
                "message": "Income deleted successfully",
                "deleted": {
                    "name": name,
                    "data": income
                }
            }));
        } else {
            return HttpResponse::NotFound().json(json!({
                "error": format!("Income '{}' not found in month {}", name, month)
            }));
        }
    }

    HttpResponse::NotFound().json(json!({
        "error": format!("Month '{}' not found", month)
    }))
}
