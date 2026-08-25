export function getYandexRequestLanguage(language) {
    return language === 'zh_tw' ? 'zh' : language;
}

export function isYandexTraditionalTarget(language) {
    return language === 'zh_tw';
}
