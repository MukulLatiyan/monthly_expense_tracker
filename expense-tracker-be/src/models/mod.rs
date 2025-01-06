use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Expense {
    pub amount: f64,
    pub paid: bool,
    #[serde(default)]
    pub date_paid: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Income {
    pub amount: f64,
    pub received: bool,
    #[serde(default)]
    pub date_received: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MonthData {
    pub expenses: HashMap<String, Expense>,
    pub income: HashMap<String, Income>,
}

#[derive(Deserialize, Debug)]
pub struct AddExpenseRequest {
    pub name: String,
    pub amount: f64,
}

#[derive(Deserialize, Debug)]
pub struct AddIncomeRequest {
    pub name: String,
    pub amount: f64,
}

#[derive(Deserialize, Debug)]
pub struct UpdateAmountRequest {
    pub amount: f64,
}
