import { fetch } from '@tauri-apps/api/http';

import {
    buildMyMemoryQuery,
    inferMyMemoryLanguage,
    MYMEMORY_ENDPOINT,
    normalizeMyMemoryLanguage,
    splitMyMemoryText,
    parseMyMemoryResponse,
} from './request';

export async function translate(text, from, to, options = {}) {
    const { config = {}, detect = '' } = options;
    const source =
        from === 'auto'
            ? !detect || normalizeMyMemoryLanguage(detect) === 'auto'
                ? inferMyMemoryLanguage(text)
                : normalizeMyMemoryLanguage(detect)
            : from;
    const translations = [];
    for (const chunk of splitMyMemoryText(text)) {
        const content = chunk.trim();
        if (content === '') {
            translations.push(chunk);
            continue;
        }

        const query = buildMyMemoryQuery(content, source, to, config.email ?? '');
        const response = await fetch(MYMEMORY_ENDPOINT, {
            method: 'GET',
            query,
        });

        if (!response.ok) {
            throw new Error(`MyMemory HTTP ${response.status}: ${JSON.stringify(response.data)}`);
        }
        const leading = chunk.slice(0, chunk.length - chunk.trimStart().length);
        const trailing = chunk.slice(chunk.trimEnd().length);
        translations.push(`${leading}${parseMyMemoryResponse(response.data)}${trailing}`);
    }
    return translations.join('');
}

export * from './Config';
export * from './info';
