<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import Editor from '$lib/components/Editor.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import ThemePicker from '$lib/components/ThemePicker.svelte';
	import {
		getAllNotes,
		getNote,
		saveNote,
		deleteNote,
		createNote,
		titleFromContent,
		type Note
	} from '$lib/storage/notes';

	let notes: Note[] = $state([]);
	let activeNote: Note | null = $state(null);
	let content = $state('');
	let theme = $state('monokai');
	let sidebarOpen = $state(true);
	let saveTimer: ReturnType<typeof setTimeout>;
	let shared = $state(false);

	onMount(async () => {
		const saved = localStorage.getItem('calpad-theme');
		if (saved) theme = saved;

		const sidebarPref = localStorage.getItem('calpad-sidebar');
		if (sidebarPref !== null) sidebarOpen = sidebarPref !== 'false';

		// Check for shared content in URL hash
		const hash = window.location.hash.slice(1);
		if (hash) {
			try {
				const decoded = atob(hash);
				const note = createNote();
				note.content = decoded;
				note.title = titleFromContent(decoded);
				await saveNote(note);
				window.location.hash = '';
			} catch {
				// Invalid hash, ignore
			}
		}

		notes = await getAllNotes();
		if (notes.length === 0) {
			const note = createNote();
			note.content = WELCOME_CONTENT;
			note.title = titleFromContent(WELCOME_CONTENT);
			await saveNote(note);
			notes = [note];
		}
		await selectNote(notes[0].id);
	});

	const WELCOME_CONTENT = `# Welcome to Calpad

Price: $10
Tax: 8.25%
$10 + 8.25%

sqrt 144
2 ^ 10
pi * 2

1 km in miles
100 celsius in fahrenheit
5 kg + 500 g
20 ml in teaspoons

now
fromunix(0) + 365 days

// Try your own calculations!`;

	async function selectNote(id: string) {
		const note = await getNote(id);
		if (note) {
			activeNote = note;
			content = note.content;
		}
	}

	function handleContentChange() {
		if (!activeNote) return;
		activeNote.content = content;
		activeNote.title = titleFromContent(content);
		activeNote.updatedAt = Date.now();

		clearTimeout(saveTimer);
		saveTimer = setTimeout(async () => {
			if (activeNote) {
				await saveNote(activeNote);
				notes = await getAllNotes();
			}
		}, 300);
	}

	async function handleCreate() {
		const note = createNote();
		await saveNote(note);
		notes = await getAllNotes();
		await selectNote(note.id);
	}

	async function handleDelete(id: string) {
		await deleteNote(id);
		notes = await getAllNotes();
		if (notes.length === 0) {
			await handleCreate();
		} else if (activeNote?.id === id) {
			await selectNote(notes[0].id);
		}
	}

	function handleThemeChange(t: string) {
		theme = t;
		localStorage.setItem('calpad-theme', t);
	}

	function toggleSidebar() {
		sidebarOpen = !sidebarOpen;
		localStorage.setItem('calpad-sidebar', String(sidebarOpen));
	}

	async function shareNote() {
		if (!activeNote || !browser) return;
		const encoded = btoa(activeNote.content);
		const url = `${window.location.origin}${window.location.pathname}#${encoded}`;
		await navigator.clipboard.writeText(url);
		shared = true;
		setTimeout(() => (shared = false), 2000);
	}

	function handleGlobalKeydown(e: KeyboardEvent) {
		const mod = e.metaKey || e.ctrlKey;
		if (mod && e.key === 'b') {
			e.preventDefault();
			toggleSidebar();
		}
		if (mod && e.key === 'n') {
			e.preventDefault();
			handleCreate();
		}
	}
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<svelte:head>
	<title>{activeNote?.title || 'Calpad'}</title>
</svelte:head>

<div class="app" data-theme={theme}>
	{#if sidebarOpen}
		<Sidebar
			{notes}
			activeId={activeNote?.id ?? ''}
			onselect={selectNote}
			oncreate={handleCreate}
			ondelete={handleDelete}
		/>
	{/if}

	<main>
		<Editor bind:content onchange={handleContentChange} />
		<footer>
			<div class="footer-left">
				<button onclick={toggleSidebar} title="Toggle sidebar (Ctrl+B)">
					{sidebarOpen ? '◀' : '▶'}
				</button>
			</div>
			<div class="footer-right">
				<button onclick={shareNote} title="Copy share link">
					{shared ? 'Copied!' : 'Share'}
				</button>
				<ThemePicker current={theme} onchange={handleThemeChange} />
			</div>
		</footer>
	</main>
</div>

<style>
	.app {
		display: flex;
		flex: 1;
		height: 100%;
		background: var(--bg);
	}

	main {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.5rem 1rem;
		border-top: 1px solid var(--border);
	}

	.footer-left, .footer-right {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}
</style>
