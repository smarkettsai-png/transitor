import { fetch } from '@tauri-apps/api/http';
import { getLingvaRequestPaths } from '../../lingva/endpoints';

export async function translate(text, from, to) {
    const plain_text = text.replaceAll('/', '@@');
    const encode_text = encodeURIComponent(plain_text);
    let lastError;

    for (const requestPath of getLingvaRequestPaths()) {
        try {
            const res = await fetch(`${requestPath}/api/v1/${from}/${to}/${encode_text}`, {
                method: 'GET',
            });

            if (res.ok) {
                const { translation } = res.data;
                if (translation) {
                    return translation.replaceAll('@@', '/');
                }
                lastError = new Error(JSON.stringify(res.data));
            } else {
                lastError = new Error(`Http Request Error\nHttp Status: ${res.status}\n${JSON.stringify(res.data)}`);
            }
        } catch (error) {
            lastError = error;
        }
    }

    throw lastError ?? new Error('Lingva translation request failed');
}

export * from './Config';
export * from './info';
