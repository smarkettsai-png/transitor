import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

import { formatEcdictResult } from '../src/services/translate/ecdict/format.js';
import { addAvailableTranslateServices, TATOEBA_PLUGIN_ID } from '../src/utils/service_activation.js';
import { getSpeechLanguage, getSpeechText } from '../src/utils/translate_result.js';
import { getSelectedText } from '../src/utils/selection_text.js';

const defaultServices = ['deepl', 'bing', 'lingva', 'yandex', 'google', 'mymemory'];
const withLocalServices = addAvailableTranslateServices(defaultServices, {
    ecdict: true,
    tatoeba: true,
});
assert.deepEqual(withLocalServices, ['ecdict', ...defaultServices, TATOEBA_PLUGIN_ID]);
assert.deepEqual(addAvailableTranslateServices(withLocalServices, { ecdict: true, tatoeba: true }), withLocalServices);

const dictionary = formatEcdictResult({
    word: 'hello',
    phonetic: 'həˈləʊ',
    translation: 'int. 你好, 喂',
    definition: 'int. greeting',
    exchange: 'p:helloed',
    tag: 'zk',
});
assert.deepEqual(dictionary.pronunciations, [{ symbol: '/həˈləʊ/' }]);
assert.equal(dictionary.explanations[0].trait, 'int');
assert.deepEqual(dictionary.explanations[0].explains, ['你好', '喂']);
assert.ok(dictionary.associations.includes('過去式: helloed'));
assert.equal(getSpeechText(dictionary), 'hello');
assert.equal(getSpeechText({ sentence: [{ source: 'Hello', target: '你好<br>您好' }] }), '你好 您好');
assert.equal(
    getSelectedText({
        activeElement: { value: 'hello world', selectionStart: 6, selectionEnd: 11 },
        getSelection: () => ({ toString: () => '' }),
    }),
    'world'
);
assert.equal(
    getSelectedText({
        activeElement: {},
        getSelection: () => ({ toString: () => '你好' }),
    }),
    '你好'
);
assert.equal(
    getSpeechLanguage(dictionary, { sourceLanguage: 'auto', detectLanguage: 'en', targetLanguage: 'zh_cn' }),
    'en'
);
assert.equal(
    getSpeechLanguage(
        { sentence: [{ source: 'Hello', target: '你好' }] },
        {
            sourceLanguage: 'en',
            detectLanguage: 'en',
            targetLanguage: 'zh_cn',
        }
    ),
    'zh_cn'
);

const pathsSource = await readFile(new URL('../src/utils/paths.js', import.meta.url), 'utf8');
assert.equal(pathsSource.includes('historyDatabasePath'), false);
assert.equal(pathsSource.includes('sqlite:'), false);
const targetAreaSource = await readFile(
    new URL('../src/window/Translate/components/TargetArea/index.jsx', import.meta.url),
    'utf8'
);
assert.match(targetAreaSource, /\/\* speak button \*\/[\s\S]*?isDisabled=\{resultSpeechText === ''\}/);
const sourceAreaSource = await readFile(
    new URL('../src/window/Translate/components/SourceArea/index.jsx', import.meta.url),
    'utf8'
);
assert.match(sourceAreaSource, /listen\('selection_translate'/);
const windowSource = await readFile(new URL('../src-tauri/src/window.rs', import.meta.url), 'utf8');
assert.match(windowSource, /is_focused\(\)[\s\S]*selection_translate/);

console.log('Portable translation regressions: PASS');
