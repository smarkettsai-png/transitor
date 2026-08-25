const DEEPL_ONESHOT_LANGUAGE = {
    EN: 'en-US',
    'ZH-HANS': 'zh-Hans',
    'ZH-HANT': 'zh-Hant',
    JA: 'ja',
    KO: 'ko',
    FR: 'fr',
    ES: 'es',
    RU: 'ru',
    DE: 'de',
    IT: 'it',
    TR: 'tr',
    'PT-PT': 'pt-PT',
    'PT-BR': 'pt-BR',
    ID: 'id',
    SV: 'sv',
    PL: 'pl',
    NL: 'nl',
    UK: 'uk',
};

export function getDeepLOneshotLanguage(language) {
    return DEEPL_ONESHOT_LANGUAGE[language] ?? language.toLowerCase();
}
