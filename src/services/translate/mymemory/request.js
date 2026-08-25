export const MYMEMORY_ENDPOINT = 'https://api.mymemory.translated.net/get';
export const MYMEMORY_MAX_QUERY_BYTES = 500;

function byteLength(value) {
    return new TextEncoder().encode(value).length;
}

const LANGUAGE_ALIASES = {
    auto: 'auto',
    zh: 'zh-CN',
    zh_cn: 'zh-CN',
    zh_tw: 'zh-TW',
    'zh-CN': 'zh-CN',
    'zh-TW': 'zh-TW',
    pt: 'pt-PT',
    pt_pt: 'pt-PT',
    pt_br: 'pt-BR',
    'pt-PT': 'pt-PT',
    'pt-BR': 'pt-BR',
    nb_no: 'nb',
    nn_no: 'nn',
    mn_cy: 'mn',
};

export function normalizeMyMemoryLanguage(language) {
    return LANGUAGE_ALIASES[language] ?? language;
}

export function inferMyMemoryLanguage(text) {
    if (/[\u4e00-\u9fff]/.test(text)) {
        return 'zh-CN';
    }
    if (/[\u3040-\u30ff]/.test(text)) {
        return 'ja';
    }
    if (/[\uac00-\ud7af]/.test(text)) {
        return 'ko';
    }
    if (/[\u0400-\u04ff]/.test(text)) {
        return 'ru';
    }
    return 'en';
}

export function splitMyMemoryText(text, maxBytes = MYMEMORY_MAX_QUERY_BYTES) {
    if (byteLength(text) <= maxBytes) {
        return [text];
    }

    const chunks = [];
    let current = '';
    const tokens = text.match(/\s+|\S+/gu) ?? [text];
    for (const token of tokens) {
        if (byteLength(token) > maxBytes) {
            if (current !== '') {
                chunks.push(current);
                current = '';
            }
            for (const character of token) {
                if (byteLength(current + character) > maxBytes) {
                    chunks.push(current);
                    current = '';
                }
                current += character;
            }
            continue;
        }
        if (current !== '' && byteLength(current + token) > maxBytes) {
            chunks.push(current);
            current = '';
        }
        current += token;
    }
    if (current !== '') {
        chunks.push(current);
    }
    return chunks;
}

export function buildMyMemoryQuery(text, from, to, email = '') {
    const source = normalizeMyMemoryLanguage(from);
    const target = normalizeMyMemoryLanguage(to);
    if (!source || source === 'auto') {
        throw new Error('MyMemory requires a detected source language');
    }
    if (!target || target === 'auto') {
        throw new Error('MyMemory requires a target language');
    }

    const query = {
        q: text,
        langpair: `${source}|${target}`,
    };
    if (email.trim() !== '') {
        query.de = email.trim();
    }
    return query;
}

export function parseMyMemoryResponse(data) {
    if (Number(data?.responseStatus) !== 200 || !data?.responseData?.translatedText) {
        const details = data?.responseDetails || 'Unknown MyMemory response';
        throw new Error(`MyMemory request failed: ${details}`);
    }
    return data.responseData.translatedText.trim();
}
