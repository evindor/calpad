<script lang="ts">
	const themes = [
		{ id: 'monokai', name: 'Monokai' },
		{ id: 'dracula', name: 'Dracula' },
		{ id: 'nord', name: 'Nord' },
		{ id: 'one-dark', name: 'One Dark' },
		{ id: 'catppuccin', name: 'Catppuccin' },
		{ id: 'gruvbox', name: 'Gruvbox' },
		{ id: 'solarized-dark', name: 'Solarized Dark' },
		{ id: 'solarized-light', name: 'Solarized Light' }
	];

	let { current, onchange }: { current: string; onchange: (theme: string) => void } = $props();
	let open = $state(false);
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="backdrop" onclick={() => (open = false)} onkeydown={() => {}}></div>
	<div class="picker">
		{#each themes as theme}
			<button
				class="theme-option"
				class:active={theme.id === current}
				onclick={() => {
					onchange(theme.id);
					open = false;
				}}
			>
				{theme.name}
			</button>
		{/each}
	</div>
{/if}

<button class="trigger" onclick={() => (open = !open)} title="Change theme">
	<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
		<circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.5" />
		<path d="M8 2a6 6 0 0 0 0 12z" fill="currentColor" />
	</svg>
</button>

<style>
	.trigger {
		position: relative;
	}

	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 10;
	}

	.picker {
		position: absolute;
		bottom: 100%;
		right: 0;
		margin-bottom: 0.5rem;
		background: var(--bg-sidebar);
		border: 1px solid var(--border);
		border-radius: 6px;
		padding: 0.25rem;
		z-index: 11;
		min-width: 160px;
	}

	.theme-option {
		width: 100%;
		text-align: left;
		padding: 0.4rem 0.75rem;
		border-radius: 4px;
		font-size: 13px;
	}

	.theme-option:hover {
		background: var(--bg-hover);
	}

	.theme-option.active {
		background: var(--bg-active);
		color: var(--fg);
	}
</style>
