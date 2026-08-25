import assert from 'node:assert/strict';

import {
    buildMyMemoryQuery,
    MYMEMORY_ENDPOINT,
    MYMEMORY_MAX_QUERY_BYTES,
    parseMyMemoryResponse,
    splitMyMemoryText,
} from '../src/services/translate/mymemory/request.js';

const query = buildMyMemoryQuery('Good morning', 'en', 'zh-TW');
assert.deepEqual(query, {
    q: 'Good morning',
    langpair: 'en|zh-TW',
});
assert.deepEqual(buildMyMemoryQuery('Hello', 'en', 'zh-TW', 'test@example.com'), {
    q: 'Hello',
    langpair: 'en|zh-TW',
    de: 'test@example.com',
});
assert.throws(() => buildMyMemoryQuery('Hello', 'auto', 'zh-TW'), /detected source language/);
const chunks = splitMyMemoryText(`${'word '.repeat(140)}終`, MYMEMORY_MAX_QUERY_BYTES);
assert.ok(chunks.length > 1);
assert.ok(chunks.every((chunk) => new TextEncoder().encode(chunk).length <= MYMEMORY_MAX_QUERY_BYTES));
assert.equal(parseMyMemoryResponse({ responseStatus: 200, responseData: { translatedText: '早安' } }), '早安');

const url = new URL(MYMEMORY_ENDPOINT);
for (const [key, value] of Object.entries(query)) {
    url.searchParams.set(key, value);
}
const response = await fetch(url);
assert.equal(response.ok, true, `MyMemory HTTP ${response.status}`);
const data = await response.json();
assert.equal(Number(data.responseStatus), 200, JSON.stringify(data));
assert.ok(data.responseData?.translatedText, JSON.stringify(data));

console.log(`MyMemory endpoint test: PASS (${data.responseData.translatedText})`);
