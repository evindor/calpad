<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { browser } from "$app/environment";
    import Editor from "$lib/components/Editor.svelte";
    import Sidebar from "$lib/components/Sidebar.svelte";
    import ThemePicker from "$lib/components/ThemePicker.svelte";
    import { onCurrencyStatus } from "$lib/engine/calpad";
    import {
        getAllNotes,
        getNote,
        saveNote,
        deleteNote,
        createNote,
        titleFromContent,
        createGuideNote,
        GUIDE_NOTE_ID,
        type Note,
    } from "$lib/storage/notes";

    let currencyLoading = $state(false);
    const unsubCurrency = onCurrencyStatus(
        (loading) => (currencyLoading = loading),
    );
    onDestroy(unsubCurrency);

    let notes: Note[] = $state([]);
    let guideNote: Note | null = $state(null);
    let activeNote: Note | null = $state(null);
    let content = $state("");
    let theme = $state("monokai");
    let sidebarOpen = $state(true);
    let saveTimer: ReturnType<typeof setTimeout>;
    let shared = $state(false);

    async function loadNotes() {
        const all = await getAllNotes();
        guideNote = all.find((n) => n.id === GUIDE_NOTE_ID) ?? null;
        notes = all.filter((n) => n.id !== GUIDE_NOTE_ID);
    }

    async function ensureGuideNote() {
        if (guideNote) return;
        const res = await fetch(`/guide.txt?at=${+Date.now()}`);
        const text = await res.text();
        const note = createGuideNote(text);
        await saveNote(note);
        guideNote = note;
    }

    async function resetGuide() {
        const res = await fetch(`/guide.txt?at=${+Date.now()}`);
        const text = await res.text();
        const note = createGuideNote(text);
        await saveNote(note);
        guideNote = note;
        if (activeNote?.id === GUIDE_NOTE_ID) {
            activeNote = note;
            content = note.content;
        }
    }

    onMount(async () => {
        const saved = localStorage.getItem("calpad-theme");
        if (saved) theme = saved;

        const sidebarPref = localStorage.getItem("calpad-sidebar");
        if (sidebarPref !== null) sidebarOpen = sidebarPref !== "false";

        // Check for shared content in URL hash
        const hash = window.location.hash.slice(1);
        if (hash) {
            try {
                const decoded = atob(hash);
                const note = createNote();
                note.content = decoded;
                note.title = titleFromContent(decoded);
                await saveNote(note);
                window.location.hash = "";
            } catch {
                // Invalid hash, ignore
            }
        }

        await loadNotes();
        await ensureGuideNote();

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

        // Snapshot to a plain object — IndexedDB can't clone $state proxies
        const snapshot = { ...activeNote };

        clearTimeout(saveTimer);
        saveTimer = setTimeout(async () => {
            await saveNote(snapshot);
            await loadNotes();
        }, 300);
    }

    async function handleCreate() {
        const note = createNote();
        await saveNote(note);
        await loadNotes();
        await selectNote(note.id);
    }

    async function handleDelete(id: string) {
        await deleteNote(id);
        await loadNotes();
        if (notes.length === 0) {
            await handleCreate();
        } else if (activeNote?.id === id) {
            await selectNote(notes[0].id);
        }
    }

    function handleThemeChange(t: string) {
        theme = t;
        localStorage.setItem("calpad-theme", t);
    }

    function toggleSidebar() {
        sidebarOpen = !sidebarOpen;
        localStorage.setItem("calpad-sidebar", String(sidebarOpen));
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
        if (mod && e.key === "b") {
            e.preventDefault();
            toggleSidebar();
        }
        if (mod && e.key === "n") {
            e.preventDefault();
            handleCreate();
        }
    }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<svelte:head>
    <title>{activeNote?.title || "Calpad"}</title>
</svelte:head>

<div class="app" data-theme={theme}>
    <div class="sidebar-slot" class:collapsed={!sidebarOpen}>
        <Sidebar
            {notes}
            activeId={activeNote?.id ?? ""}
            {guideNote}
            onselect={selectNote}
            oncreate={handleCreate}
            ondelete={handleDelete}
            onresetguide={resetGuide}
        />
    </div>

    <main>
        <Editor bind:content onchange={handleContentChange} />
        <footer>
            <div class="footer-left">
                <button
                    class="toggle-sidebar"
                    class:open={sidebarOpen}
                    onclick={toggleSidebar}
                    title="Toggle sidebar (Ctrl+B)"
                >
                    <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                        <path
                            d="M5 2.5l4.5 4.5-4.5 4.5"
                            stroke="currentColor"
                            stroke-width="1.5"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        />
                    </svg>
                </button>
            </div>
            <div class="footer-right">
                {#if currencyLoading}
                    <span class="currency-loading"
                        >Updating currency rates...</span
                    >
                {/if}
                <button onclick={shareNote} title="Copy share link">
                    {shared ? "Copied!" : "Share"}
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

    .sidebar-slot {
        width: var(--sidebar-width);
        flex-shrink: 0;
        overflow: hidden;
        transition: width 0.15s ease;
        align-self: stretch;
    }

    .sidebar-slot.collapsed {
        width: 0;
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

    .footer-left,
    .footer-right {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .toggle-sidebar {
        display: flex;
        align-items: center;
        justify-content: center;
        transition: transform 0.15s ease;
        transform: rotate(0deg);
    }

    .toggle-sidebar.open {
        transform: rotate(180deg);
    }

    .currency-loading {
        font-size: 12px;
        color: var(--fg-muted);
    }
</style>
