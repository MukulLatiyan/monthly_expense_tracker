use crate::state::AppState;
use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;

#[get("/debug")]
pub async fn debug_data(data: web::Data<AppState>) -> impl Responder {
    let data = data.data.lock().unwrap();
    HttpResponse::Ok().json(&*data)
}

#[get("/months/{month}/summary")]
pub async fn get_summary(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let month = path.into_inner();
    let data = data.data.lock().unwrap();

    if let Some(month_data) = data.get(&month) {
        let total_expenses: f64 = month_data.expenses.values().map(|e| e.amount).sum();
        let total_income: f64 = month_data.income.values().map(|i| i.amount).sum();
        let received_income: f64 = month_data
            .income
            .values()
            .filter(|i| i.received)
            .map(|i| i.amount)
            .sum();
        let pending_income = total_income - received_income;
        let balance = total_income - total_expenses;

        let summary = json!({
            "total_income": total_income,
            "received_income": received_income,
            "pending_income": pending_income,
            "total_expenses": total_expenses,
            "balance": balance,
            "spending_ratio": if total_income > 0.0 { (total_expenses / total_income) * 100.0 } else { 0.0 }
        });

        HttpResponse::Ok().json(summary)
    } else {
        HttpResponse::NotFound().finish()
    }
}
