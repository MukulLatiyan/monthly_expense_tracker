const inr = new Intl.NumberFormat("en-IN", {
  style: "currency",
  currency: "INR",
  maximumFractionDigits: 2,
  minimumFractionDigits: 0,
});

export function formatInr(value: unknown): string {
  // Handle all edge cases safely
  if (value === null || value === undefined) return "₹0";
  
  let num: number;
  if (typeof value === "number") {
    num = value;
  } else if (typeof value === "string") {
    num = parseFloat(value);
  } else {
    num = Number(value);
  }
  
  if (Number.isNaN(num) || !Number.isFinite(num)) {
    return "₹0";
  }
  
  return inr.format(num);
}

export const MONTH_NAMES = [
  "Jan", "Feb", "Mar", "Apr", "May", "Jun",
  "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"
];

// Backend expects MM-YYYY format (e.g., "04-2026")
export function currentMonthId(): string {
  const d = new Date();
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  return `${m}-${y}`;
}

export function parseMonthId(monthId: string): { year: number; month: number } {
  const parts = monthId.split("-");
  // Backend format is MM-YYYY
  const m = parts[0] ?? "01";
  const y = parts[1] ?? "2024";
  return { year: parseInt(y, 10), month: parseInt(m, 10) };
}

export function formatMonthId(monthId: string): string {
  const { year, month } = parseMonthId(monthId);
  return `${MONTH_NAMES[month - 1]} ${year}`;
}

export function makeMonthId(year: number, month: number): string {
  // Backend format: MM-YYYY
  return `${String(month).padStart(2, "0")}-${year}`;
}

export function getYearRange(): number[] {
  const currentYear = new Date().getFullYear();
  // Range: 2 years back to 2 years forward
  const years: number[] = [];
  for (let y = currentYear - 2; y <= currentYear + 2; y++) {
    years.push(y);
  }
  return years;
}

export function parseAmountInput(raw: string): number | null {
  if (!raw || typeof raw !== "string") return null;
  const cleaned = raw.replace(/,/g, "").trim();
  if (!cleaned) return null;
  const n = Number(cleaned);
  if (Number.isNaN(n) || n < 0 || !Number.isFinite(n)) return null;
  return n;
}
