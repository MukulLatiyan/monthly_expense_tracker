export type ApiResponse<T> = {
  success: boolean;
  data: T;
  message: string;
  error?: string;
};

export type Expense = {
  month: string;
  name: string;
  amount: number;
  paid: boolean;
  date_paid?: string;
  created_at: string;
};

export type Income = {
  month: string;
  name: string;
  amount: number;
  received: boolean;
  date_received?: string;
  created_at: string;
};

export type Summary = {
  month: string;
  total_income: number;
  received_income: number;
  pending_income: number;
  total_expenses: number;
  paid_expenses: number;
  balance: number;
  actual_balance: number;
  expense_count: number;
  income_count: number;
};
