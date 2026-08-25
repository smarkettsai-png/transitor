const EXCHANGE_LABELS = {
    p: '過去式',
    d: '過去分詞',
    i: '現在分詞',
    3: '第三人稱單數',
    r: '比較級',
    t: '最高級',
    s: '複數',
    0: 'Lemma',
    1: 'Lemma',
};

function formatExplanations(translation, definition) {
    const source = translation?.trim() || definition?.trim() || '';
    if (source === '') {
        return [];
    }

    return source.split(/\r?\n/).map((line) => {
        const separator = line.indexOf('.');
        const trait = separator > 0 ? line.slice(0, separator).trim() : '';
        const explanation = separator > 0 ? line.slice(separator + 1) : line;
        return {
            trait,
            explains: explanation
                .split(',')
                .map((value) => value.trim())
                .filter(Boolean),
        };
    });
}

function formatAssociations(exchange, tag) {
    const associations = [];
    if (exchange) {
        for (const item of exchange.split('/')) {
            const separator = item.indexOf(':');
            if (separator <= 0) {
                continue;
            }
            const label = EXCHANGE_LABELS[item.slice(0, separator)];
            const word = item.slice(separator + 1).trim();
            if (label && word) {
                associations.push(`${label}: ${word}`);
            }
        }
    }

    if (tag) {
        associations.push('');
        associations.push(tag);
    }
    return associations;
}

export function formatEcdictResult(row) {
    const target = {
        speechText: row.word?.trim() || '',
        speechLanguage: 'source',
        explanations: formatExplanations(row.translation, row.definition),
    };
    if (row.phonetic) {
        target.pronunciations = [{ symbol: `/${row.phonetic}/` }];
    }

    const associations = formatAssociations(row.exchange, row.tag);
    if (associations.length > 0) {
        target.associations = associations;
    }
    return target;
}
