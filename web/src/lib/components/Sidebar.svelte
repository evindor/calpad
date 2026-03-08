<script lang="ts">
    type Note = {
        id: string;
        title: string;
        updatedAt: number;
    };

    let {
        notes,
        activeId,
        guideNote,
        onselect,
        oncreate,
        ondelete,
        onresetguide,
    }: {
        notes: Note[];
        activeId: string;
        guideNote: Note | null;
        onselect: (id: string) => void;
        oncreate: () => void;
        ondelete: (id: string) => void;
        onresetguide: () => void;
    } = $props();

    let confirmDeleteId: string | null = $state(null);

    function titleFor(note: Note): string {
        return note.title || "Untitled";
    }

    function formatDate(ts: number): string {
        const d = new Date(ts);
        const now = new Date();
        if (d.toDateString() === now.toDateString()) {
            return d.toLocaleTimeString([], {
                hour: "2-digit",
                minute: "2-digit",
            });
        }
        return d.toLocaleDateString([], { month: "short", day: "numeric" });
    }

    function handleDelete(e: MouseEvent, id: string) {
        e.stopPropagation();
        if (confirmDeleteId === id) {
            ondelete(id);
            confirmDeleteId = null;
        } else {
            confirmDeleteId = id;
            setTimeout(() => {
                if (confirmDeleteId === id) confirmDeleteId = null;
            }, 3000);
        }
    }
</script>

<aside class="sidebar">
    <div class="sidebar-header">
        <span class="logo">calpad</span>
        <button onclick={oncreate} title="New note (Ctrl+N)">+</button>
    </div>
    <nav class="notes-list">
        {#each notes as note (note.id)}
            <div class="note-row" class:active={note.id === activeId}>
                <button class="note-item" onclick={() => onselect(note.id)}>
                    <span class="note-title">{titleFor(note)}</span>
                    <span class="note-date"
                        >{formatDate(note.updatedAt)}</span
                    >
                </button>
                <button
                    class="note-delete"
                    class:confirm={confirmDeleteId === note.id}
                    onclick={(e) => handleDelete(e, note.id)}
                    title={confirmDeleteId === note.id
                        ? "Click again to confirm"
                        : "Delete note"}
                >
                    {confirmDeleteId === note.id ? "?" : "\u00D7"}
                </button>
            </div>
        {/each}
    </nav>
    {#if guideNote}
        <div class="guide-section">
            <div class="note-row" class:active={guideNote.id === activeId}>
                <button
                    class="note-item"
                    onclick={() => onselect(guideNote.id)}
                >
                    <span class="note-title">Guide</span>
                </button>
                <button
                    class="note-reset"
                    onclick={(e) => {
                        e.stopPropagation();
                        onresetguide();
                    }}
                    title="Reset to default"
                >
                    <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                        <path
                            d="M2 8a6 6 0 0 1 10.3-4.15L14 2v5h-5l1.8-1.8A4.5 4.5 0 1 0 12.5 8"
                            stroke="currentColor"
                            stroke-width="1.5"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        />
                    </svg>
                </button>
            </div>
        </div>
    {/if}
</aside>

<style>
    .sidebar {
        width: var(--sidebar-width);
        height: 100%;
        background: var(--bg-sidebar);
        border-right: 1px solid var(--border);
        display: flex;
        flex-direction: column;
        flex-shrink: 0;
        overflow: hidden;
    }

    .sidebar-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 1rem;
        border-bottom: 1px solid var(--border);
    }

    .logo {
        font-weight: 700;
        font-size: 13px;
        letter-spacing: 0.05em;
        color: var(--fg-muted);
    }

    .sidebar-header button {
        font-size: 18px;
        width: 24px;
        height: 24px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 4px;
    }

    .sidebar-header button:hover {
        background: var(--bg-hover);
    }

    .notes-list {
        flex: 1;
        overflow-y: auto;
        padding: 0.5rem;
    }

    .note-row {
        display: flex;
        align-items: center;
        border-radius: 4px;
        position: relative;
    }

    .note-row:hover {
        background: var(--bg-hover);
    }

    .note-row.active {
        background: var(--bg-active);
    }

    .note-item {
        flex: 1;
        display: flex;
        align-items: baseline;
        justify-content: space-between;
        gap: 0.5rem;
        padding: 0.5rem 0.75rem;
        text-align: left;
        font-size: 13px;
        min-width: 0;
    }

    .note-row.active .note-item {
        color: var(--fg);
    }

    .note-title {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        flex: 1;
    }

    .note-date {
        color: var(--fg-muted);
        font-size: 11px;
        flex-shrink: 0;
    }

    .note-delete {
        opacity: 0;
        font-size: 14px;
        width: 20px;
        height: 20px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 3px;
        flex-shrink: 0;
        margin-right: 4px;
    }

    .note-row:hover .note-delete {
        opacity: 0.5;
    }

    .note-delete:hover {
        opacity: 1 !important;
        background: var(--bg-hover);
    }

    .note-delete.confirm {
        opacity: 1 !important;
        color: var(--color-error);
    }

    .guide-section {
        padding: 0 0.5rem 0.5rem;
        flex-shrink: 0;
        position: relative;
    }

    .guide-section::before {
        content: '';
        position: absolute;
        bottom: 100%;
        left: 0;
        right: 0;
        height: 2rem;
        background: linear-gradient(to bottom, transparent, var(--bg-sidebar));
        pointer-events: none;
    }

    .note-reset {
        opacity: 0;
        width: 20px;
        height: 20px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 3px;
        flex-shrink: 0;
        margin-right: 4px;
    }

    .guide-section .note-row:hover .note-reset {
        opacity: 0.5;
    }

    .note-reset:hover {
        opacity: 1 !important;
        background: var(--bg-hover);
    }
</style>
