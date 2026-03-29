import { useCallback, useEffect, useMemo, useState } from "react";
import * as api from "./api";
import { 
  currentMonthId, 
  formatInr, 
  parseAmountInput,
  MONTH_NAMES,
  parseMonthId,
  makeMonthId,
  getYearRange,
} from "./format";
import type { Expense, Income, Summary } from "./types";

type Mode = "income" | "expense";

const DEFAULT_SUMMARY: Summary = {
  month: currentMonthId(),
  total_income: 0,
  received_income: 0,
  pending_income: 0,
  total_expenses: 0,
  paid_expenses: 0,
  balance: 0,
  actual_balance: 0,
  expense_count: 0,
  income_count: 0,
};

export default function App() {
  const [month, setMonth] = useState(currentMonthId);
  const [summary, setSummary] = useState<Summary>(DEFAULT_SUMMARY);
  const [income, setIncome] = useState<Income[]>([]);
  const [expenses, setExpenses] = useState<Expense[]>([]);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [actionSuccess, setActionSuccess] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);

  const [editKey, setEditKey] = useState<string | null>(null);
  const [editValue, setEditValue] = useState("");

  const [addIncomeName, setAddIncomeName] = useState("");
  const [addIncomeAmount, setAddIncomeAmount] = useState("");
  const [addExpenseName, setAddExpenseName] = useState("");
  const [addExpenseAmount, setAddExpenseAmount] = useState("");

  // Parse current month/year for the pickers
  const { year: currentYear, month: currentMonth } = useMemo(() => parseMonthId(month), [month]);
  const availableYears = useMemo(() => getYearRange(), []);

  const refresh = useCallback(async () => {
    setLoadError(null);
    setLoading(true);
    try {
      const [s, inc, exp] = await Promise.all([
        api.getSummary(month),
        api.getIncome(month),
        api.getExpenses(month),
      ]);
      setSummary(s ?? DEFAULT_SUMMARY);
      setIncome(inc ?? []);
      setExpenses(exp ?? []);
    } catch (e) {
      setSummary(DEFAULT_SUMMARY);
      setIncome([]);
      setExpenses([]);
      setLoadError(e instanceof Error ? e.message : "Failed to load data");
    } finally {
      setLoading(false);
    }
  }, [month]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    if (actionError) {
      const t = setTimeout(() => setActionError(null), 5000);
      return () => clearTimeout(t);
    }
  }, [actionError]);

  useEffect(() => {
    if (actionSuccess) {
      const t = setTimeout(() => setActionSuccess(null), 3000);
      return () => clearTimeout(t);
    }
  }, [actionSuccess]);

  async function run(fn: () => Promise<unknown>, successMsg?: string): Promise<boolean> {
    setActionError(null);
    setActionSuccess(null);
    setBusy(true);
    try {
      await fn();
      if (successMsg) setActionSuccess(successMsg);
      return true;
    } catch (e) {
      setActionError(e instanceof Error ? e.message : "Request failed");
      return false;
    } finally {
      setBusy(false);
    }
  }

  function startEdit(mode: Mode, name: string, amount: number) {
    setEditKey(`${mode}:${name}`);
    setEditValue(String(amount));
  }

  function cancelEdit() {
    setEditKey(null);
    setEditValue("");
  }

  async function saveEdit(mode: Mode, name: string) {
    const amt = parseAmountInput(editValue);
    if (amt === null) {
      setActionError("Enter a valid amount");
      return;
    }
    const ok =
      mode === "expense"
        ? await run(() => api.updateExpenseAmount(month, name, amt), "Expense amount updated")
        : await run(() => api.updateIncomeAmount(month, name, amt), "Income amount updated");
    if (ok) {
      cancelEdit();
      await refresh();
    }
  }

  async function toggleIncome(item: Income) {
    const msg = item.received ? "Income marked as pending" : "Income marked as received";
    const ok = await run(() => 
      item.received
        ? api.markIncomeUnreceived(month, item.name)
        : api.markIncomeReceived(month, item.name),
      msg
    );
    if (ok) await refresh();
  }

  async function toggleExpense(item: Expense) {
    const msg = item.paid ? "Expense marked as unpaid" : "Expense marked as paid";
    const ok = await run(() =>
      item.paid
        ? api.markExpenseUnpaid(month, item.name)
        : api.markExpensePaid(month, item.name),
      msg
    );
    if (ok) await refresh();
  }

  async function removeIncome(name: string) {
    if (!window.confirm(`Remove income "${name}"?`)) return;
    const ok = await run(() => api.deleteIncome(month, name), "Income removed");
    if (ok) await refresh();
  }

  async function removeExpense(name: string) {
    if (!window.confirm(`Remove expense "${name}"?`)) return;
    const ok = await run(() => api.deleteExpense(month, name), "Expense removed");
    if (ok) await refresh();
  }

  async function submitIncome() {
    const amt = parseAmountInput(addIncomeAmount);
    if (!addIncomeName.trim()) {
      setActionError("Income name is required");
      return;
    }
    if (amt === null) {
      setActionError("Income amount is invalid");
      return;
    }
    const ok = await run(() =>
      api.addIncome(month, { name: addIncomeName.trim(), amount: amt }),
      "Income added"
    );
    if (ok) {
      setAddIncomeName("");
      setAddIncomeAmount("");
      await refresh();
    }
  }

  async function submitExpense() {
    const amt = parseAmountInput(addExpenseAmount);
    if (!addExpenseName.trim()) {
      setActionError("Expense name is required");
      return;
    }
    if (amt === null) {
      setActionError("Expense amount is invalid");
      return;
    }
    const ok = await run(() =>
      api.addExpense(month, { name: addExpenseName.trim(), amount: amt }),
      "Expense added"
    );
    if (ok) {
      setAddExpenseName("");
      setAddExpenseAmount("");
      await refresh();
    }
  }

  return (
    <div className="app">
      <header className="header">
        <h1 className="title">Expense Tracker</h1>
        <div className="month-picker">
          <label className="sr-only" htmlFor="month">Month</label>
          <select
            id="month"
            className="month-select"
            value={currentMonth}
            onChange={(e) => setMonth(makeMonthId(currentYear, parseInt(e.target.value, 10)))}
            disabled={loading || busy}
            aria-label="Month"
          >
            {MONTH_NAMES.map((name, idx) => (
              <option key={name} value={idx + 1}>{name}</option>
            ))}
          </select>
          <label className="sr-only" htmlFor="year">Year</label>
          <select
            id="year"
            className="month-select year-select"
            value={currentYear}
            onChange={(e) => setMonth(makeMonthId(parseInt(e.target.value, 10), currentMonth))}
            disabled={loading || busy}
            aria-label="Year"
          >
            {availableYears.map((y) => (
              <option key={y} value={y}>{y}</option>
            ))}
          </select>
        </div>
      </header>

      {loadError && (
        <div className="banner error" role="alert">
          Error: {loadError}
        </div>
      )}
      {actionError && (
        <div className="banner error" role="alert">
          {actionError}
        </div>
      )}
      {actionSuccess && (
        <div className="banner success" role="status">
          {actionSuccess}
        </div>
      )}
      {loading && (
        <div className="loading-container">
          <div className="spinner" />
          <span>Loading data...</span>
        </div>
      )}

      <section className={`summary ${busy ? "dimmed" : ""}`} aria-busy={busy}>
        <div className="stat">
          <label>Balance (Planned)</label>
          <div className={`num ${summary.balance >= 0 ? "pos" : "neg"}`}>
            {formatInr(summary.balance)}
          </div>
        </div>
        <div className="stat">
          <label>Actual Balance</label>
          <div className={`num ${summary.actual_balance >= 0 ? "pos" : "neg"}`}>
            {formatInr(summary.actual_balance)}
          </div>
        </div>
        <div className="stat">
          <label>Total Income</label>
          <div className="num">{formatInr(summary.total_income)}</div>
        </div>
        <div className="stat">
          <label>Total Expenses</label>
          <div className="num neg">{formatInr(summary.total_expenses)}</div>
        </div>
      </section>

      <section className={`section ${busy ? "dimmed" : ""}`}>
        <h2 className="section-title">Income</h2>
        <div className="table-container">
          <div className="table-scroll">
            <table className="ledger">
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Status</th>
                  <th className="num">Amount</th>
                </tr>
              </thead>
              <tbody>
                {income.length === 0 ? (
                  <tr>
                    <td colSpan={3} className="empty-row">
                      No income recorded for this month. Add your first income below.
                    </td>
                  </tr>
                ) : (
                  income.map((row) => (
                    <IncomeRow
                      key={`income-${row.name}`}
                      row={row}
                      editKey={editKey}
                      editValue={editValue}
                      onEditChange={setEditValue}
                      onStartEdit={() => startEdit("income", row.name, row.amount)}
                      onSaveEdit={() => void saveEdit("income", row.name)}
                      onCancelEdit={cancelEdit}
                      onToggle={() => void toggleIncome(row)}
                      onDelete={() => void removeIncome(row.name)}
                      busy={busy}
                    />
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
        <div className="add-bar">
          <input
            type="text"
            placeholder="Income name"
            value={addIncomeName}
            onChange={(e) => setAddIncomeName(e.target.value)}
            disabled={busy}
            autoComplete="off"
          />
          <input
            type="text"
            inputMode="decimal"
            placeholder="Amount (e.g. 50000)"
            value={addIncomeAmount}
            onChange={(e) => setAddIncomeAmount(e.target.value)}
            disabled={busy}
          />
          <button
            type="button"
            className="btn"
            disabled={busy}
            onClick={() => void submitIncome()}
          >
            Add Income
          </button>
        </div>
      </section>

      <section className={`section ${busy ? "dimmed" : ""}`}>
        <h2 className="section-title">Expenses</h2>
        <div className="table-container">
          <div className="table-scroll">
            <table className="ledger">
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Status</th>
                  <th className="num">Amount</th>
                </tr>
              </thead>
              <tbody>
                {expenses.length === 0 ? (
                  <tr>
                    <td colSpan={3} className="empty-row">
                      No expenses recorded for this month. Add your first expense below.
                    </td>
                  </tr>
                ) : (
                  expenses.map((row) => (
                    <ExpenseRow
                      key={`expense-${row.name}`}
                      row={row}
                      editKey={editKey}
                      editValue={editValue}
                      onEditChange={setEditValue}
                      onStartEdit={() => startEdit("expense", row.name, row.amount)}
                      onSaveEdit={() => void saveEdit("expense", row.name)}
                      onCancelEdit={cancelEdit}
                      onToggle={() => void toggleExpense(row)}
                      onDelete={() => void removeExpense(row.name)}
                      busy={busy}
                    />
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
        <div className="add-bar">
          <input
            type="text"
            placeholder="Expense name"
            value={addExpenseName}
            onChange={(e) => setAddExpenseName(e.target.value)}
            disabled={busy}
            autoComplete="off"
          />
          <input
            type="text"
            inputMode="decimal"
            placeholder="Amount (e.g. 25000)"
            value={addExpenseAmount}
            onChange={(e) => setAddExpenseAmount(e.target.value)}
            disabled={busy}
          />
          <button
            type="button"
            className="btn"
            disabled={busy}
            onClick={() => void submitExpense()}
          >
            Add Expense
          </button>
        </div>
      </section>

      <footer className="footer">
        Configure <code>VITE_API_BASE_URL</code> in <code>.env</code> for local dev or <code>.env.production</code> for deployment.
      </footer>
    </div>
  );
}

interface RowProps {
  row: Income | Expense;
  editKey: string | null;
  editValue: string;
  onEditChange: (v: string) => void;
  onStartEdit: () => void;
  onSaveEdit: () => void;
  onCancelEdit: () => void;
  onToggle: () => void;
  onDelete: () => void;
  busy: boolean;
}

function IncomeRow({
  row,
  editKey,
  editValue,
  onEditChange,
  onStartEdit,
  onSaveEdit,
  onCancelEdit,
  onToggle,
  onDelete,
  busy,
}: RowProps & { row: Income }) {
  const key = `income:${row.name}`;
  const editing = editKey === key;
  const received = Boolean(row.received);

  return (
    <>
      <tr>
        <td className="row-name" title={row.name}>{row.name}</td>
        <td>
          <span className={received ? "pill ok" : "pill no"}>
            {received ? "Received" : "Pending"}
          </span>
        </td>
        <td className="num">
          {editing ? (
            <input
              className="amount-input"
              value={editValue}
              onChange={(e) => onEditChange(e.target.value)}
              disabled={busy}
              inputMode="decimal"
              aria-label="Amount"
            />
          ) : (
            formatInr(row.amount)
          )}
        </td>
      </tr>
      <tr className="row-actions">
        <td colSpan={3}>
          <div className="actions">
            <button
              type="button"
              className="action-btn"
              disabled={busy}
              onClick={onToggle}
            >
              {received ? "Mark Pending" : "Mark Received"}
            </button>
            {editing ? (
              <>
                <button
                  type="button"
                  className="action-btn"
                  disabled={busy}
                  onClick={onSaveEdit}
                >
                  Save
                </button>
                <button
                  type="button"
                  className="action-btn"
                  disabled={busy}
                  onClick={onCancelEdit}
                >
                  Cancel
                </button>
              </>
            ) : (
              <button
                type="button"
                className="action-btn"
                disabled={busy}
                onClick={onStartEdit}
              >
                Edit Amount
              </button>
            )}
            <button
              type="button"
              className="action-btn danger"
              disabled={busy}
              onClick={onDelete}
            >
              Remove
            </button>
          </div>
        </td>
      </tr>
    </>
  );
}

function ExpenseRow({
  row,
  editKey,
  editValue,
  onEditChange,
  onStartEdit,
  onSaveEdit,
  onCancelEdit,
  onToggle,
  onDelete,
  busy,
}: RowProps & { row: Expense }) {
  const key = `expense:${row.name}`;
  const editing = editKey === key;
  const paid = Boolean(row.paid);

  return (
    <>
      <tr>
        <td className="row-name" title={row.name}>{row.name}</td>
        <td>
          <span className={paid ? "pill ok" : "pill no"}>
            {paid ? "Paid" : "Unpaid"}
          </span>
        </td>
        <td className="num">
          {editing ? (
            <input
              className="amount-input"
              value={editValue}
              onChange={(e) => onEditChange(e.target.value)}
              disabled={busy}
              inputMode="decimal"
              aria-label="Amount"
            />
          ) : (
            formatInr(row.amount)
          )}
        </td>
      </tr>
      <tr className="row-actions">
        <td colSpan={3}>
          <div className="actions">
            <button
              type="button"
              className="action-btn"
              disabled={busy}
              onClick={onToggle}
            >
              {paid ? "Mark Unpaid" : "Mark Paid"}
            </button>
            {editing ? (
              <>
                <button
                  type="button"
                  className="action-btn"
                  disabled={busy}
                  onClick={onSaveEdit}
                >
                  Save
                </button>
                <button
                  type="button"
                  className="action-btn"
                  disabled={busy}
                  onClick={onCancelEdit}
                >
                  Cancel
                </button>
              </>
            ) : (
              <button
                type="button"
                className="action-btn"
                disabled={busy}
                onClick={onStartEdit}
              >
                Edit Amount
              </button>
            )}
            <button
              type="button"
              className="action-btn danger"
              disabled={busy}
              onClick={onDelete}
            >
              Remove
            </button>
          </div>
        </td>
      </tr>
    </>
  );
}
