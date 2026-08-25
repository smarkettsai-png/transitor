function stripMarkup(value) {
    return String(value)
        .replace(/<br\s*\/?>/gi, ' ')
        .replace(/<[^>]*>/g, '')
        .replace(/&nbsp;/gi, ' ')
        .replace(/&amp;/gi, '&')
        .replace(/&lt;/gi, '<')
        .replace(/&gt;/gi, '>')
        .replace(/&quot;/gi, '"')
        .replace(/&#39;|&#x27;/gi, "'")
        .replace(/\s+/g, ' ')
        .trim();
}

export function getSpeechText(result) {
    if (typeof result === 'string') {
        return result.trim();
    }
    if (!result || typeof result !== 'object') {
        return '';
    }
    if (typeof result.speechText === 'string' && result.speechText.trim() !== '') {
        return result.speechText.trim();
    }

    const sentences = Array.isArray(result.sentence) ? result.sentence : [];
    for (const sentence of sentences) {
        const target = stripMarkup(sentence?.target ?? '');
        if (target !== '') {
            return target;
        }
    }
    for (const sentence of sentences) {
        const source = stripMarkup(sentence?.source ?? '');
        if (source !== '') {
            return source;
        }
    }
    return '';
}

export function getSpeechLanguage(result, { sourceLanguage, detectLanguage, targetLanguage } = {}) {
    if (result?.speechLanguage === 'source') {
        return sourceLanguage === 'auto' ? detectLanguage : sourceLanguage;
    }
    return targetLanguage;
}
