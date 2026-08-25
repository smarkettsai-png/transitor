import { fetch } from '@tauri-apps/api/http';
import { getLingvaRequestPaths } from '../../lingva/endpoints';

export async function tts(text, lang, options = {}) {
    const { config } = options;
    let lastError;

    for (const requestPath of getLingvaRequestPaths(config?.requestPath)) {
        try {
            const res = await fetch(`${requestPath}/api/v1/audio/${lang}/${encodeURIComponent(text)}`);
            if (res.ok && res.data?.audio) {
                return res.data.audio;
            }
            lastError = new Error(`Lingva request failed with HTTP status ${res.status}`);
        } catch (error) {
            lastError = error;
        }
    }

    throw lastError ?? new Error('Lingva TTS request failed');
}

export * from './Config';
export * from './info';
