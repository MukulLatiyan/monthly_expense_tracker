use crate::handlers::{expense, general, income};
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // General routes
        .service(general::debug_data)
        .service(general::get_summary)
        // Expense routes
        .service(expense::get_expenses)
        .service(expense::add_expense)
        .service(expense::mark_expense_paid)
        .service(expense::mark_expense_unpaid)
        .service(expense::update_expense_amount)
        .service(expense::delete_expense)
        // Income routes
        .service(income::get_income)
        .service(income::add_income)
        .service(income::mark_income_received)
        .service(income::mark_income_unreceived)
        .service(income::update_income_amount)
        .service(income::delete_income);
}
