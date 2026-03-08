export interface Note {
	id: string;
	title: string;
	content: string;
	createdAt: number;
	updatedAt: number;
}

const DB_NAME = 'calpad';
const DB_VERSION = 1;
const STORE_NAME = 'notes';

function openDb(): Promise<IDBDatabase> {
	return new Promise((resolve, reject) => {
		const req = indexedDB.open(DB_NAME, DB_VERSION);
		req.onupgradeneeded = () => {
			const db = req.result;
			if (!db.objectStoreNames.contains(STORE_NAME)) {
				const store = db.createObjectStore(STORE_NAME, { keyPath: 'id' });
				store.createIndex('updatedAt', 'updatedAt');
			}
		};
		req.onsuccess = () => resolve(req.result);
		req.onerror = () => reject(req.error);
	});
}

function tx(mode: IDBTransactionMode): Promise<IDBObjectStore> {
	return openDb().then((db) => db.transaction(STORE_NAME, mode).objectStore(STORE_NAME));
}

function req<T>(r: IDBRequest<T>): Promise<T> {
	return new Promise((resolve, reject) => {
		r.onsuccess = () => resolve(r.result);
		r.onerror = () => reject(r.error);
	});
}

export async function getAllNotes(): Promise<Note[]> {
	const store = await tx('readonly');
	const all = await req(store.index('updatedAt').getAll());
	return all.reverse();
}

export async function getNote(id: string): Promise<Note | undefined> {
	const store = await tx('readonly');
	return req(store.get(id));
}

export async function saveNote(note: Note): Promise<void> {
	const store = await tx('readwrite');
	await req(store.put(note));
}

export async function deleteNote(id: string): Promise<void> {
	const store = await tx('readwrite');
	await req(store.delete(id));
}

export function createNote(): Note {
	return {
		id: crypto.randomUUID(),
		title: '',
		content: '',
		createdAt: Date.now(),
		updatedAt: Date.now()
	};
}

export const GUIDE_NOTE_ID = 'calpad-guide';

export function createGuideNote(content: string): Note {
	return {
		id: GUIDE_NOTE_ID,
		title: 'Calpad Guide',
		content,
		createdAt: 0,
		updatedAt: 0
	};
}

export function titleFromContent(content: string): string {
	const firstLine = content.split('\n').find((l) => l.trim().length > 0);
	if (!firstLine) return '';
	// Strip formatting prefixes
	return firstLine.replace(/^(#|\/\/)\s*/, '').replace(/^[\w\s]+:\s*/, '').trim().slice(0, 60);
}
