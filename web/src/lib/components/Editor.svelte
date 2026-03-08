<script lang="ts">
	import { onMount } from 'svelte';
	import { initCalpad, evaluateDocument, type LineResult } from '$lib/engine/calpad';

	let { content = $bindable(''), onchange }: { content: string; onchange?: () => void } = $props();

	let results: LineResult[] = $state([]);
	let ready = $state(false);
	let textarea: HTMLTextAreaElement;
	let editorEl: HTMLDivElement;
	let lines = $derived(content.split('\n'));

	onMount(async () => {
		await initCalpad();
		ready = true;
		doEvaluate();
		setTimeout(() => doEvaluate(), 2000);
	});

	function doEvaluate() {
		if (!ready) return;
		results = evaluateDocument(content);
	}

	function handleInput(e: Event) {
		const target = e.target as HTMLTextAreaElement;
		content = target.value;
		doEvaluate();
		onchange?.();
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Tab') {
			e.preventDefault();
			const target = e.target as HTMLTextAreaElement;
			const start = target.selectionStart;
			const end = target.selectionEnd;
			content = content.substring(0, start) + '  ' + content.substring(end);
			requestAnimationFrame(() => {
				target.selectionStart = target.selectionEnd = start + 2;
			});
			doEvaluate();
		}
	}

	function handleScroll() {
		if (editorEl && textarea) {
			editorEl.scrollTop = textarea.scrollTop;
		}
	}

	function getResultForLine(index: number): LineResult | undefined {
		return results.find((r) => r.line_index === index);
	}

	// Syntax highlighting: classify a line for coloring
	function classifyLine(line: string): 'header' | 'comment' | 'normal' {
		const trimmed = line.trim();
		if (trimmed.startsWith('#')) return 'header';
		if (trimmed.startsWith('//')) return 'comment';
		return 'normal';
	}

	// Simple syntax coloring: highlight keywords, operators, numbers, units in a line
	function highlightLine(line: string): string {
		if (!line) return '\u00A0';
		const trimmed = line.trim();
		if (trimmed.startsWith('#')) return `<span class="hl-header">${escapeHtml(line)}</span>`;
		if (trimmed.startsWith('//')) return `<span class="hl-comment">${escapeHtml(line)}</span>`;

		// Highlight parts
		return line.replace(
			/(".*?")|(\$[\d,.]+|\€[\d,.]+|\£[\d,.]+|[\d,.]+\s*(?:USD|EUR|GBP|CAD|JPY|AUD|CHF|CNY|INR|RUB|BRL|KRW|MXN|SGD|HKD|SEK|NOK|DKK|PLN|THB|NZD|ZAR|TWD|CZK|ILS)\b)|((?:0[xXbBoO])?[\d,.]+%?)|\b(in|to|as|into|of|on|off|plus|minus|times|mod|xor|and|with|without)\b|\b(sum|total|average|avg|prev|now|time)\b|\b(sqrt|cbrt|abs|round|ceil|floor|ln|log|fact|sin|cos|tan|arcsin|arccos|arctan|sinh|cosh|tanh|root|fromunix)\b|\b(kg|km|cm|mm|meter|meters|mile|miles|foot|feet|inch|inches|yard|yards|pound|pounds|ounce|ounces|gram|grams|liter|liters|gallon|gallons|celsius|fahrenheit|kelvin|degrees?|radians?|px|pt|em|byte|bytes|bit|bits|[KMGT]B|[KMGT]iB|seconds?|minutes?|hours?|days?|weeks?|months?|years?|hectare|acre|pint|quart|cup|teaspoons?|tablespoons?|tsp|tbsp)\b|([+\-*/^&|<>]+)/g,
			(match, quoted, currency, number, keyword, special, func, unit, op) => {
				if (quoted) return `<span class="hl-label">${escapeHtml(quoted)}</span>`;
				if (currency) return `<span class="hl-number">${escapeHtml(currency)}</span>`;
				if (number) return `<span class="hl-number">${escapeHtml(number)}</span>`;
				if (keyword) return `<span class="hl-keyword">${escapeHtml(keyword)}</span>`;
				if (special) return `<span class="hl-keyword">${escapeHtml(special)}</span>`;
				if (func) return `<span class="hl-unit">${escapeHtml(func)}</span>`;
				if (unit) return `<span class="hl-unit">${escapeHtml(unit)}</span>`;
				if (op) return `<span class="hl-operator">${escapeHtml(op)}</span>`;
				return escapeHtml(match);
			}
		);
	}

	function escapeHtml(s: string): string {
		return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
	}

	// Handle label coloring: "Label: rest"
	function highlightWithLabel(line: string): string {
		const trimmed = line.trim();
		if (trimmed.startsWith('#') || trimmed.startsWith('//')) return highlightLine(line);
		const colonMatch = line.match(/^([A-Za-z][A-Za-z ]*?):\s/);
		if (colonMatch) {
			const label = colonMatch[0];
			const rest = line.slice(label.length);
			return `<span class="hl-label">${escapeHtml(label)}</span>${highlightLine(rest)}`;
		}
		return highlightLine(line);
	}
</script>

<div class="editor">
	<div class="lines-layer" bind:this={editorEl} aria-hidden="true">
		{#each lines as line, i}
			{@const result = getResultForLine(i)}
			<div class="line">
				<span class="line-input">{@html highlightWithLabel(line)}</span>
				{#if result?.display}
					<span class="line-result" class:error={result.is_error}>
						{result.display}
					</span>
				{/if}
			</div>
		{/each}
	</div>
	<textarea
		bind:this={textarea}
		value={content}
		oninput={handleInput}
		onkeydown={handleKeyDown}
		onscroll={handleScroll}
		spellcheck="false"
		autocomplete="off"
		autocorrect="off"
		autocapitalize="off"
		placeholder="Start typing..."
	></textarea>
</div>

<style>
	.editor {
		flex: 1;
		position: relative;
		overflow: hidden;
	}

	textarea {
		position: absolute;
		inset: 0;
		padding: 2rem;
		width: 100%;
		height: 100%;
		background: transparent;
		color: transparent;
		caret-color: var(--fg);
		border: none;
		outline: none;
		resize: none;
		font-family: var(--font-mono);
		font-size: var(--font-size);
		line-height: var(--line-height);
		white-space: pre;
		overflow-y: auto;
		z-index: 1;
	}

	textarea::placeholder {
		color: var(--fg-muted);
	}

	.lines-layer {
		position: absolute;
		inset: 0;
		padding: 2rem;
		pointer-events: none;
		white-space: pre;
		overflow-y: auto;
	}

	.line {
		display: flex;
		justify-content: space-between;
		gap: 2rem;
		line-height: var(--line-height);
		min-height: calc(var(--font-size) * var(--line-height));
	}

	.line-input {
		flex: 1;
		color: var(--fg);
		overflow: hidden;
	}

	.line-result {
		color: var(--color-result);
		flex-shrink: 0;
		text-align: right;
		font-weight: 500;
	}

	.line-result.error {
		color: var(--color-error);
	}

	/* Syntax highlighting */
	.line-input :global(.hl-number) { color: var(--color-number); }
	.line-input :global(.hl-unit) { color: var(--color-unit); }
	.line-input :global(.hl-keyword) { color: var(--color-keyword); }
	.line-input :global(.hl-operator) { color: var(--color-operator); }
	.line-input :global(.hl-label) { color: var(--color-label); }
	.line-input :global(.hl-header) { color: var(--color-header); font-weight: 700; }
	.line-input :global(.hl-comment) { color: var(--color-comment); font-style: italic; }
</style>
