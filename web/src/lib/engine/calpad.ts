import init, { evaluate, set_currency_rates } from "./pkg/calpad_wasm.js";

let initialized = false;

export interface LineResult {
  line_index: number;
  input: string;
  display: string;
  is_error: boolean;
  raw_number: number | null;
}

const CACHE_KEY = "calpad-currency-rates";

type StatusListener = (loading: boolean) => void;
let listeners: StatusListener[] = [];

export function onCurrencyStatus(cb: StatusListener): () => void {
  listeners.push(cb);
  return () => {
    listeners = listeners.filter((l) => l !== cb);
  };
}

export async function initCalpad(): Promise<void> {
  if (initialized) return;
  await init();
  initialized = true;

  // Load cached rates synchronously so first evaluate has them
  try {
    const raw = localStorage.getItem(CACHE_KEY);
    if (raw) {
      const { rates } = JSON.parse(raw);
      set_currency_rates(rates);
    }
  } catch {}

  // Fetch fresh rates in background
  refreshRates();
}

async function refreshRates(): Promise<void> {
  listeners.forEach((cb) => cb(true));
  try {
    const response = await fetch("https://api.frankfurter.app/latest?from=USD");
    if (!response.ok) return;
    const data = await response.json();
    const rates: Record<string, number> = { USD: 1.0, ...data.rates };
    localStorage.setItem(
      CACHE_KEY,
      JSON.stringify({ rates, timestamp: Date.now() }),
    );
    set_currency_rates(rates);
  } catch {
  } finally {
    listeners.forEach((cb) => cb(false));
  }
}

export function evaluateDocument(document: string): LineResult[] {
  try {
    return evaluate(document) as LineResult[];
  } catch (e) {
    console.error(e);
    return document.split("\n").map((line, i) => ({
      line_index: i,
      input: line,
      display: "",
      is_error: false,
      raw_number: null,
    }));
  }
}
