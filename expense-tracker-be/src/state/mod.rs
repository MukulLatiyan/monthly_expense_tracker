use crate::models::{Expense, ExpenseItem, Income, IncomeItem, MonthData};
use aws_sdk_dynamodb::{error::ProvideErrorMetadata, types::AttributeValue, Client};
use serde_json;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("DynamoDB error: {0}")]
    DynamoDb(String),
    #[error("Item not found: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Extract detailed error message from AWS SDK error
fn extract_error_details<E>(err: &E) -> String
where
    E: ProvideErrorMetadata + std::fmt::Display,
{
    let mut details = vec![];
    
    if let Some(code) = err.code() {
        details.push(format!("ErrorCode: {}", code));
    }
    if let Some(msg) = err.message() {
        details.push(format!("Message: {}", msg));
    }
    
    if details.is_empty() {
        err.to_string()
    } else {
        details.join(" | ")
    }
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub table_name: String,
}

impl AppState {
    pub fn new(client: Client, table_name: String) -> Self {
        Self { client, table_name }
    }

    // Helper to create attribute value
    fn attr_s(value: impl Into<String>) -> AttributeValue {
        AttributeValue::S(value.into())
    }

    fn attr_n(value: f64) -> AttributeValue {
        AttributeValue::N(value.to_string())
    }

    fn attr_bool(value: bool) -> AttributeValue {
        AttributeValue::Bool(value)
    }

    // ==================== EXPENSE OPERATIONS ====================

    pub async fn get_expenses(&self, month: &str) -> RepositoryResult<Vec<ExpenseItem>> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("pk = :pk AND begins_with(sk, :sk_prefix)")
            .expression_attribute_values(":pk", Self::attr_s(format!("MONTH#{}", month)))
            .expression_attribute_values(":sk_prefix", Self::attr_s("EXPENSE#"))
            .send()
            .await
            .map_err(|e| {
                let details = extract_error_details(&e);
                eprintln!("DynamoDB Query Error: {}", details);
                RepositoryError::DynamoDb(details)
            })?;

        let mut expenses = Vec::new();

        if let Some(items) = result.items {
            for item in items {
                let expense = self.parse_expense_item(&item, month)?;
                expenses.push(expense);
            }
        }

        Ok(expenses)
    }

    pub async fn add_expense(
        &self,
        month: &str,
        name: &str,
        amount: f64,
    ) -> RepositoryResult<ExpenseItem> {
        // Check if expense already exists
        if let Ok(Some(_)) = self.get_expense_by_name(month, name).await {
            return Err(RepositoryError::Validation(format!(
                "Expense '{}' already exists for month {}",
                name, month
            )));
        }

        let expense = Expense {
            amount,
            paid: false,
            date_paid: None,
        };

        let item = serde_json::to_string(&expense)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        self.client
            .put_item()
            .table_name(&self.table_name)
            .item("pk", Self::attr_s(format!("MONTH#{}", month)))
            .item("sk", Self::attr_s(format!("EXPENSE#{}", name)))
            .item("month", Self::attr_s(month))
            .item("name", Self::attr_s(name))
            .item("type", Self::attr_s("expense"))
            .item("amount", Self::attr_n(amount))
            .item("paid", Self::attr_bool(false))
            .item("data", Self::attr_s(item))
            .send()
            .await
            .map_err(|e| {
                let details = extract_error_details(&e);
                eprintln!("DynamoDB Query Error: {}", details);
                RepositoryError::DynamoDb(details)
            })?;

        Ok(ExpenseItem {
            month: month.to_string(),
            name: name.to_string(),
            data: expense,
        })
    }

    pub async fn get_expense_by_name(
        &self,
        month: &str,
        name: &str,
    ) -> RepositoryResult<Option<ExpenseItem>> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("pk", Self::attr_s(format!("MONTH#{}", month)))
            .key("sk", Self::attr_s(format!("EXPENSE#{}", name)))
            .send()
            .await
            .map_err(|e| {
                let details = extract_error_details(&e);
                eprintln!("DynamoDB Query Error: {}", details);
                RepositoryError::DynamoDb(details)
            })?;

        if let Some(item) = result.item {
            Ok(Some(self.parse_expense_item(&item, month)?))
        } else {
            Ok(None)
        }
    }

    pub async fn mark_expense_paid(&self, month: &str, name: &str) -> RepositoryResult<ExpenseItem> {
        let expense = self
            .get_expense_by_name(month, name)
            .await?
            .ok_or_else(|| {
                RepositoryError::NotFound(format!(
                    "Expense '{}' not found for month {}",
                    name, month
                ))
            })?;

        let now = chrono::Local::now().to_string();
        let updated_expense = Expense {
            amount: expense.data.amount,
            paid: true,
            date_paid: Some(now.clone()),
        };

        let item = serde_json::to_string(&updated_expense)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("pk", Self::attr_s(format!("MONTH#{}", month)))
            .key("sk", Self::attr_s(format!("EXPENSE#{}", name)))
            .update_expression("SET paid = :paid, date_paid = :date_paid, #data = :data")
            .expression_attribute_names("#data", "data")
            .expression_attribute_values(":paid", Self::attr_bool(true))
            .expression_attribute_values(":date_paid", Self::attr_s(now))
            .expression_attribute_values(":data", Self::attr_s(item))
            .send()
            .await
            .map_err(|e| {
                let details = extract_error_details(&e);
                eprintln!("DynamoDB Query Error: {}", details);
                RepositoryError::DynamoDb(details)
            })?;

        Ok(ExpenseItem {
            month: month.to_string(),
            name: name.to_string(),
            data: updated_expense,
        })
    }

    pub async fn mark_expense_unpaid(
        &self,
        month: &str,
        name: &str,
    ) -> RepositoryResult<ExpenseItem> {
        let expense = self
            .get_expense_by_name(month, name)
            .await?
            .ok_or_else(|| {
                RepositoryError::NotFound(format!(
                    "Expense '{}' not found for month {}",
                    name, month
                ))
            })?;

        let updated_expense = Expense {
            amount: expense.data.amount,
            paid: false,
            date_paid: None,
        };

        let item = serde_json::to_string(&updated_expense)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("pk", Self::attr_s(format!("MONTH#{}", month)))
            .key("sk", Self::attr_s(format!("EXPENSE#{}", name)))
            .update_expression("SET paid = :paid, date_paid = :date_paid, #data = :data")
            .expression_attribute_names("#data", "data")
            .expression_attribute_values(":paid", Self::attr_bool(false))
            .expression_attribute_values(":date_paid", AttributeValue::Null(true))
            .expression_attribute_values(":data", Self::attr_s(item))
            .send()
            .await
            .map_err(|e| {
                let details = extract_error_details(&e);
                eprintln!("DynamoDB Query Error: {}", details);
                RepositoryError::DynamoDb(details)
            })?;

        Ok(ExpenseItem {
            month: month.to_string(),
            name: name.to_string(),
            data: updated_expense,
        })
    }

    pub async fn update_expense_amount(
        &self,
        month: &str,
        name: &str,
        amount: f64,
    ) -> RepositoryResult<ExpenseItem> {
        let expense = self
            .get_expense_by_name(month, name)
            .await?
            .ok_or_else(|| {
                RepositoryError::NotFound(format!(
                    "Expense '{}' not found for month {}",
                    name, month
                ))
            })?;

        let updated_expense = Expense {
            amount,
            paid: expense.data.paid,
            date_paid: expense.data.date_paid,
        };

        let item = serde_json::to_string(&updated_expense)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("pk", Self::attr_s(format!("MONTH#{}", month)))
            .key("sk", Self::attr_s(format!("EXPENSE#{}", name)))
            .update_expression("SET amount = :amount, #data = :data")
            .expression_attribute_names("#data", "data")
            .expression_attribute_values(":amount", Self::attr_n(amount))
            .expression_attribute_values(":data", Self::attr_s(item))
            .send()
            .await
            .map_err(|e| {
                let details = extract_error_details(&e);
                eprintln!("DynamoDB Query Error: {}", details);
                RepositoryError::DynamoDb(details)
            })?;

        Ok(ExpenseItem {
            month: month.to_string(),
            name: name.to_string(),
            data: updated_expense,
        })
    }

    pub async fn delete_expense(&self, month: &str, name: &str) -> RepositoryResult<ExpenseItem> {
        let expense = self
            .get_expense_by_name(month, name)
            .await?
            .ok_or_else(|| {
                RepositoryError::NotFound(format!(
                    "Expense '{}' not found for month {}",
                    name, month
                ))
            })?;

        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("pk", Self::attr_s(format!("MONTH#{}", month)))
            .key("sk", Self::attr_s(format!("EXPENSE#{}", name)))
            .send()
            .await
            .map_err(|e| {
                let details = extract_error_details(&e);
                eprintln!("DynamoDB Query Error: {}", details);
                RepositoryError::DynamoDb(details)
            })?;

        Ok(expense)
    }

    // ==================== INCOME OPERATIONS ====================

    pub async fn get_income(&self, month: &str) -> RepositoryResult<Vec<IncomeItem>> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("pk = :pk AND begins_with(sk, :sk_prefix)")
            .expression_attribute_values(":pk", Self::attr_s(format!("MONTH#{}", month)))
            .expression_attribute_values(":sk_prefix", Self::attr_s("INCOME#"))
            .send()
            .await
            .map_err(|e| {
                let details = extract_error_details(&e);
                eprintln!("DynamoDB Query Error: {}", details);
                RepositoryError::DynamoDb(details)
            })?;

        let mut income = Vec::new();

        if let Some(items) = result.items {
            for item in items {
                let income_item = self.parse_income_item(&item, month)?;
                income.push(income_item);
            }
        }

        Ok(income)
    }

    pub async fn add_income(
        &self,
        month: &str,
        name: &str,
        amount: f64,
    ) -> RepositoryResult<IncomeItem> {
        // Check if income already exists
        if let Ok(Some(_)) = self.get_income_by_name(month, name).await {
            return Err(RepositoryError::Validation(format!(
                "Income '{}' already exists for month {}",
                name, month
            )));
        }

        let income = Income {
            amount,
            received: false,
            date_received: None,
        };

        let item = serde_json::to_string(&income)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        self.client
            .put_item()
            .table_name(&self.table_name)
            .item("pk", Self::attr_s(format!("MONTH#{}", month)))
            .item("sk", Self::attr_s(format!("INCOME#{}", name)))
            .item("month", Self::attr_s(month))
            .item("name", Self::attr_s(name))
            .item("type", Self::attr_s("income"))
            .item("amount", Self::attr_n(amount))
            .item("received", Self::attr_bool(false))
            .item("data", Self::attr_s(item))
            .send()
            .await
            .map_err(|e| {
                let details = extract_error_details(&e);
                eprintln!("DynamoDB Query Error: {}", details);
                RepositoryError::DynamoDb(details)
            })?;

        Ok(IncomeItem {
            month: month.to_string(),
            name: name.to_string(),
            data: income,
        })
    }

    pub async fn get_income_by_name(
        &self,
        month: &str,
        name: &str,
    ) -> RepositoryResult<Option<IncomeItem>> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("pk", Self::attr_s(format!("MONTH#{}", month)))
            .key("sk", Self::attr_s(format!("INCOME#{}", name)))
            .send()
            .await
            .map_err(|e| {
                let details = extract_error_details(&e);
                eprintln!("DynamoDB Query Error: {}", details);
                RepositoryError::DynamoDb(details)
            })?;

        if let Some(item) = result.item {
            Ok(Some(self.parse_income_item(&item, month)?))
        } else {
            Ok(None)
        }
    }

    pub async fn mark_income_received(
        &self,
        month: &str,
        name: &str,
    ) -> RepositoryResult<IncomeItem> {
        let income = self
            .get_income_by_name(month, name)
            .await?
            .ok_or_else(|| {
                RepositoryError::NotFound(format!(
                    "Income '{}' not found for month {}",
                    name, month
                ))
            })?;

        let now = chrono::Local::now().to_string();
        let updated_income = Income {
            amount: income.data.amount,
            received: true,
            date_received: Some(now.clone()),
        };

        let item = serde_json::to_string(&updated_income)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("pk", Self::attr_s(format!("MONTH#{}", month)))
            .key("sk", Self::attr_s(format!("INCOME#{}", name)))
            .update_expression("SET received = :received, date_received = :date_received, #data = :data")
            .expression_attribute_names("#data", "data")
            .expression_attribute_values(":received", Self::attr_bool(true))
            .expression_attribute_values(":date_received", Self::attr_s(now))
            .expression_attribute_values(":data", Self::attr_s(item))
            .send()
            .await
            .map_err(|e| {
                let details = extract_error_details(&e);
                eprintln!("DynamoDB Query Error: {}", details);
                RepositoryError::DynamoDb(details)
            })?;

        Ok(IncomeItem {
            month: month.to_string(),
            name: name.to_string(),
            data: updated_income,
        })
    }

    pub async fn mark_income_unreceived(
        &self,
        month: &str,
        name: &str,
    ) -> RepositoryResult<IncomeItem> {
        let income = self
            .get_income_by_name(month, name)
            .await?
            .ok_or_else(|| {
                RepositoryError::NotFound(format!(
                    "Income '{}' not found for month {}",
                    name, month
                ))
            })?;

        let updated_income = Income {
            amount: income.data.amount,
            received: false,
            date_received: None,
        };

        let item = serde_json::to_string(&updated_income)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("pk", Self::attr_s(format!("MONTH#{}", month)))
            .key("sk", Self::attr_s(format!("INCOME#{}", name)))
            .update_expression("SET received = :received, date_received = :date_received, #data = :data")
            .expression_attribute_names("#data", "data")
            .expression_attribute_values(":received", Self::attr_bool(false))
            .expression_attribute_values(":date_received", AttributeValue::Null(true))
            .expression_attribute_values(":data", Self::attr_s(item))
            .send()
            .await
            .map_err(|e| {
                let details = extract_error_details(&e);
                eprintln!("DynamoDB Query Error: {}", details);
                RepositoryError::DynamoDb(details)
            })?;

        Ok(IncomeItem {
            month: month.to_string(),
            name: name.to_string(),
            data: updated_income,
        })
    }

    pub async fn update_income_amount(
        &self,
        month: &str,
        name: &str,
        amount: f64,
    ) -> RepositoryResult<IncomeItem> {
        let income = self
            .get_income_by_name(month, name)
            .await?
            .ok_or_else(|| {
                RepositoryError::NotFound(format!(
                    "Income '{}' not found for month {}",
                    name, month
                ))
            })?;

        let updated_income = Income {
            amount,
            received: income.data.received,
            date_received: income.data.date_received,
        };

        let item = serde_json::to_string(&updated_income)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("pk", Self::attr_s(format!("MONTH#{}", month)))
            .key("sk", Self::attr_s(format!("INCOME#{}", name)))
            .update_expression("SET amount = :amount, #data = :data")
            .expression_attribute_names("#data", "data")
            .expression_attribute_values(":amount", Self::attr_n(amount))
            .expression_attribute_values(":data", Self::attr_s(item))
            .send()
            .await
            .map_err(|e| {
                let details = extract_error_details(&e);
                eprintln!("DynamoDB Query Error: {}", details);
                RepositoryError::DynamoDb(details)
            })?;

        Ok(IncomeItem {
            month: month.to_string(),
            name: name.to_string(),
            data: updated_income,
        })
    }

    pub async fn delete_income(&self, month: &str, name: &str) -> RepositoryResult<IncomeItem> {
        let income = self
            .get_income_by_name(month, name)
            .await?
            .ok_or_else(|| {
                RepositoryError::NotFound(format!(
                    "Income '{}' not found for month {}",
                    name, month
                ))
            })?;

        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("pk", Self::attr_s(format!("MONTH#{}", month)))
            .key("sk", Self::attr_s(format!("INCOME#{}", name)))
            .send()
            .await
            .map_err(|e| {
                let details = extract_error_details(&e);
                eprintln!("DynamoDB Query Error: {}", details);
                RepositoryError::DynamoDb(details)
            })?;

        Ok(income)
    }

    // ==================== SUMMARY ====================

    pub async fn get_month_summary(&self, month: &str) -> RepositoryResult<MonthData> {
        let expenses = self.get_expenses(month).await?;
        let income = self.get_income(month).await?;

        Ok(MonthData { expenses, income })
    }

    // ==================== PARSING HELPERS ====================

    fn parse_expense_item(
        &self,
        item: &HashMap<String, AttributeValue>,
        month: &str,
    ) -> RepositoryResult<ExpenseItem> {
        let name = item
            .get("name")
            .and_then(|v| v.as_s().ok())
            .map(|s| s.to_string())
            .unwrap_or_default();

        if let Some(data_attr) = item.get("data") {
            if let Ok(data_json) = data_attr.as_s() {
                let expense: Expense = serde_json::from_str(data_json)
                    .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
                return Ok(ExpenseItem {
                    month: month.to_string(),
                    name,
                    data: expense,
                });
            }
        }

        // Fallback to individual fields
        let amount = item
            .get("amount")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<f64>().ok())
            .unwrap_or(0.0);

        let paid = item
            .get("paid")
            .and_then(|v| v.as_bool().ok())
            .copied()
            .unwrap_or(false);

        let date_paid = item
            .get("date_paid")
            .and_then(|v| v.as_s().ok())
            .map(|s| s.to_string());

        Ok(ExpenseItem {
            month: month.to_string(),
            name,
            data: Expense {
                amount,
                paid,
                date_paid,
            },
        })
    }

    fn parse_income_item(
        &self,
        item: &HashMap<String, AttributeValue>,
        month: &str,
    ) -> RepositoryResult<IncomeItem> {
        let name = item
            .get("name")
            .and_then(|v| v.as_s().ok())
            .map(|s| s.to_string())
            .unwrap_or_default();

        if let Some(data_attr) = item.get("data") {
            if let Ok(data_json) = data_attr.as_s() {
                let income: Income = serde_json::from_str(data_json)
                    .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
                return Ok(IncomeItem {
                    month: month.to_string(),
                    name,
                    data: income,
                });
            }
        }

        // Fallback to individual fields
        let amount = item
            .get("amount")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<f64>().ok())
            .unwrap_or(0.0);

        let received = item
            .get("received")
            .and_then(|v| v.as_bool().ok())
            .copied()
            .unwrap_or(false);

        let date_received = item
            .get("date_received")
            .and_then(|v| v.as_s().ok())
            .map(|s| s.to_string());

        Ok(IncomeItem {
            month: month.to_string(),
            name,
            data: Income {
                amount,
                received,
                date_received,
            },
        })
    }
}
