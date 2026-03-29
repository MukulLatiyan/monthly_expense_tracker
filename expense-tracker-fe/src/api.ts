import type { ApiResponse, Expense, Income, Summary } from "./types";

function getBaseUrl(): string {
  const url = import.meta.env.VITE_API_BASE_URL;
  if (!url || typeof url !== "string") {
    return "";
  }
  return url.trim().replace(/\/$/, "");
}

async function jsonFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const base = getBaseUrl();
  if (!base) {
    throw new Error(
      "Missing VITE_API_BASE_URL. Set your API URL in .env file."
    );
  }

  let res: Response;
  try {
    res = await fetch(`${base}${path}`, {
      ...init,
      headers: {
        "Content-Type": "application/json",
        ...init?.headers,
      },
    });
  } catch (networkErr) {
    throw new Error("Network error: Cannot connect to API.");
  }

  const raw = await res.text();
  let body: ApiResponse<T>;
  
  try {
    body = (raw ? JSON.parse(raw) : {}) as ApiResponse<T>;
  } catch {
    throw new Error(
      res.ok
        ? "Invalid response from server (not JSON)"
        : `HTTP ${res.status}: ${res.statusText}`
    );
  }
  
  if (!res.ok || !body.success) {
    const msg = body?.error ?? body?.message ?? `HTTP ${res.status}`;
    throw new Error(msg);
  }
  
  return body.data as T;
}

function enc(name: string): string {
  return encodeURIComponent(name);
}

// Default empty responses to prevent crashes
export async function getSummary(month: string): Promise<Summary> {
  try {
    const data = await jsonFetch<Summary>(`/months/${enc(month)}/summary`);
    if (!data || typeof data !== "object") {
      throw new Error("Invalid summary data from server");
    }
    return data;
  } catch (e) {
    console.error("getSummary error:", e);
    throw e;
  }
}

export async function getExpenses(month: string): Promise<Expense[]> {
  try {
    const data = await jsonFetch<Expense[]>(`/months/${enc(month)}/expenses`);
    if (!Array.isArray(data)) return [];
    return data.filter((item): item is Expense => 
      item && typeof item === "object" && typeof item.name === "string"
    );
  } catch (e) {
    console.error("getExpenses error:", e);
    return [];
  }
}

export async function getIncome(month: string): Promise<Income[]> {
  try {
    const data = await jsonFetch<Income[]>(`/months/${enc(month)}/income`);
    if (!Array.isArray(data)) return [];
    return data.filter((item): item is Income => 
      item && typeof item === "object" && typeof item.name === "string"
    );
  } catch (e) {
    console.error("getIncome error:", e);
    return [];
  }
}

export async function addExpense(
  month: string,
  body: { name: string; amount: number }
): Promise<Expense> {
  return jsonFetch<Expense>(`/months/${enc(month)}/expenses`, {
    method: "POST",
    body: JSON.stringify(body),
  });
}

export async function addIncome(
  month: string,
  body: { name: string; amount: number }
): Promise<Income> {
  return jsonFetch<Income>(`/months/${enc(month)}/income`, {
    method: "POST",
    body: JSON.stringify(body),
  });
}

export async function markExpensePaid(month: string, name: string): Promise<Expense> {
  return jsonFetch<Expense>(`/months/${enc(month)}/expenses/${enc(name)}/paid`, {
    method: "PUT",
  });
}

export async function markExpenseUnpaid(month: string, name: string): Promise<Expense> {
  return jsonFetch<Expense>(`/months/${enc(month)}/expenses/${enc(name)}/unpaid`, {
    method: "PUT",
  });
}

export async function markIncomeReceived(month: string, name: string): Promise<Income> {
  return jsonFetch<Income>(`/months/${enc(month)}/income/${enc(name)}/received`, {
    method: "PUT",
  });
}

export async function markIncomeUnreceived(month: string, name: string): Promise<Income> {
  return jsonFetch<Income>(`/months/${enc(month)}/income/${enc(name)}/unreceived`, {
    method: "PUT",
  });
}

export async function updateExpenseAmount(
  month: string,
  name: string,
  amount: number
): Promise<Expense> {
  return jsonFetch<Expense>(`/months/${enc(month)}/expenses/${enc(name)}/amount`, {
    method: "PUT",
    body: JSON.stringify({ amount }),
  });
}

export async function updateIncomeAmount(
  month: string,
  name: string,
  amount: number
): Promise<Income> {
  return jsonFetch<Income>(`/months/${enc(month)}/income/${enc(name)}/amount`, {
    method: "PUT",
    body: JSON.stringify({ amount }),
  });
}

export async function deleteExpense(month: string, name: string): Promise<void> {
  await jsonFetch<null>(`/months/${enc(month)}/expenses/${enc(name)}`, {
    method: "DELETE",
  });
}

export async function deleteIncome(month: string, name: string): Promise<void> {
  await jsonFetch<null>(`/months/${enc(month)}/income/${enc(name)}`, {
    method: "DELETE",
  });
}
