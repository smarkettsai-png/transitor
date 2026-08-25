import assert from 'node:assert/strict';

const { getDeepLOneshotLanguage } = await import('../src/services/translate/deepl/language.js');
const { getYandexRequestLanguage, isYandexTraditionalTarget } = await import(
    '../src/services/translate/yandex/language.js'
);
const { simplifiedToTraditional } = await import('../src/utils/chinese.js');

assert.equal(getDeepLOneshotLanguage('ZH-HANS'), 'zh-Hans');
assert.equal(getDeepLOneshotLanguage('ZH-HANT'), 'zh-Hant');
assert.equal(getYandexRequestLanguage('zh_tw'), 'zh');
assert.equal(isYandexTraditionalTarget('zh_tw'), true);
assert.equal(simplifiedToTraditional('软件和网络服务'), '軟體和網路服務');

console.log('Traditional Chinese service mapping: PASS');
