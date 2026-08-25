import { invoke } from '@tauri-apps/api/tauri';

import { formatEcdictResult } from './format';

export async function translate(text) {
    const word = text.trim();
    if (word === '') {
        return { explanations: [] };
    }

    const row = await invoke('ecdict_lookup', { word });
    if (!row) {
        throw new Error(`ECDict entry not found: ${word}`);
    }
    return formatEcdictResult(row);
}

export * from './Config';
export * from './info';
