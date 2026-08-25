import { fetch, Body } from '@tauri-apps/api/http';
import { v4 as uuidv4 } from 'uuid';

import { getYandexRequestLanguage, isYandexTraditionalTarget } from './language';
import { simplifiedToTraditional } from '../../../utils/chinese';

export async function translate(text, from, to) {
    const url = 'https://translate.yandex.net/api/v1/tr.json/translate';
    const res = await fetch(url, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
        },
        query: {
            id: uuidv4().replaceAll('-', '') + '-0-0',
            srv: 'android',
        },
        body: Body.form({
            source_lang: getYandexRequestLanguage(from),
            target_lang: getYandexRequestLanguage(to),
            text,
        }),
    });
    if (res.ok) {
        const result = res.data;
        if (result.text) {
            const translatedText = result.text[0];
            return isYandexTraditionalTarget(to) ? simplifiedToTraditional(translatedText) : translatedText;
        } else {
            throw JSON.stringify(result);
        }
    } else {
        throw `Http Request Error\nHttp Status: ${res.status}\n${JSON.stringify(res.data)}`;
    }
}

export * from './Config';
export * from './info';
