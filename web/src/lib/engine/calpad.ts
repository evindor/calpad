import init, { evaluate, set_currency_rates } from './pkg/calpad_wasm.js';

let initialized = false;

export interface LineResult {
	line_index: number;
	input: string;
	display: string;
	is_error: boolean;
}

export async function initCalpad(): Promise<void> {
	if (initialized) return;
	await init();
	initialized = true;

	// Fetch currency rates in the background (non-blocking)
	fetchCurrencyRates().catch(() => {
		// Silent fail — currencies will work without conversion
	});
}

export function evaluateDocument(document: string): LineResult[] {
	return evaluate(document) as LineResult[];
}

async function fetchCurrencyRates(): Promise<void> {
	const response = await fetch('https://api.frankfurter.app/latest?from=USD');
	if (!response.ok) return;
	const data = await response.json();
	// data.rates is { EUR: 0.92, GBP: 0.79, ... } relative to USD
	const rates: Record<string, number> = { USD: 1.0, ...data.rates };
	set_currency_rates(rates);
}
