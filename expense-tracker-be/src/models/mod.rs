use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Expense {
    pub amount: f64,
    pub paid: bool,
    #[serde(default)]
    pub date_paid: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Income {
    pub amount: f64,
    pub received: bool,
    #[serde(default)]
    pub date_received: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MonthData {
    #[serde(default)]
    pub expenses: Vec<ExpenseItem>,
    #[serde(default)]
    pub income: Vec<IncomeItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExpenseItem {
    pub month: String,
    pub name: String,
    #[serde(flatten)]
    pub data: Expense,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IncomeItem {
    pub month: String,
    pub name: String,
    #[serde(flatten)]
    pub data: Income,
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

// Response models for better API responses
#[derive(Serialize, Debug)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ExpenseListResponse {
    pub month: String,
    pub expenses: Vec<ExpenseItem>,
    pub count: usize,
}

#[derive(Serialize, Debug)]
pub struct IncomeListResponse {
    pub month: String,
    pub income: Vec<IncomeItem>,
    pub count: usize,
}

#[derive(Serialize, Debug)]
pub struct SummaryResponse {
    pub month: String,
    pub total_income: f64,
    pub received_income: f64,
    pub pending_income: f64,
    pub total_expenses: f64,
    pub balance: f64,
    pub spending_ratio: f64,
    pub expense_count: usize,
    pub income_count: usize,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.into(),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>, error_detail: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            message: message.into(),
            error: Some(error_detail.into()),
        }
    }

    pub fn empty(message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: None,
            message: message.into(),
            error: None,
        }
    }
}
