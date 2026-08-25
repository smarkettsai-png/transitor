import assert from 'node:assert/strict';
import {
    DEFAULT_LINGVA_REQUEST_PATH,
    getLingvaRequestPaths,
} from '../src/services/lingva/endpoints.js';

assert.equal(DEFAULT_LINGVA_REQUEST_PATH, 'lingva.ml');
assert.deepEqual(getLingvaRequestPaths(), [
    'https://lingva.ml',
    'https://translate.plausibility.cloud',
    'https://lingva.lunar.icu',
]);
assert.deepEqual(getLingvaRequestPaths('lingva.pot-app.com'), [
    'https://lingva.ml',
    'https://translate.plausibility.cloud',
    'https://lingva.lunar.icu',
]);
assert.deepEqual(getLingvaRequestPaths('https://lingva.pot-app.com/'), [
    'https://lingva.ml',
    'https://translate.plausibility.cloud',
    'https://lingva.lunar.icu',
]);
assert.deepEqual(getLingvaRequestPaths('https://my-lingva.example/api'), [
    'https://my-lingva.example/api',
]);

console.log('Lingva endpoint tests: PASS');
