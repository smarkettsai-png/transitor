import { readFile } from 'node:fs/promises';
import vm from 'node:vm';

const pluginPath = process.argv[2] ?? 'portable-assets/tatoeba/plugin.com.pot-app.tatoeba/main.js';
const source = await readFile(pluginPath, 'utf8');
const sandbox = {};
vm.createContext(sandbox);
vm.runInContext(`${source}\nglobalThis.pluginTranslate = translate;`, sandbox);

const result = await sandbox.pluginTranslate('hello', 'eng', 'cmn', {
    detect: 'en',
    utils: {
        tauriFetch: async (url, options) => {
            if (url !== 'https://api.tatoeba.org/v1/sentences') {
                throw new Error(`Unexpected Tatoeba URL: ${url}`);
            }
            if (
                options.query.limit !== '10' ||
                options.query['trans:lang'] !== 'cmn' ||
                options.query['showtrans:lang'] !== 'cmn'
            ) {
                throw new Error(`Unexpected Tatoeba language query: ${JSON.stringify(options.query)}`);
            }
            return {
                ok: true,
                status: 200,
                data: {
                    data: [
                        { text: 'Hello', translations: [{ text: '你好' }] },
                        { text: '<Hi>', translations: [[{ text: '嗨' }]] },
                    ],
                },
            };
        },
    },
});

if (
    result.sentence?.length !== 2 ||
    result.sentence[0].target !== '你好' ||
    result.sentence[1].source !== '&lt;Hi&gt;' ||
    result.speechText !== '你好' ||
    result.speechLanguage !== 'target'
) {
    throw new Error(`Unexpected Tatoeba result: ${JSON.stringify(result)}`);
}

console.log('Tatoeba plugin parser: PASS');
