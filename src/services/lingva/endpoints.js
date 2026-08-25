export const DEFAULT_LINGVA_REQUEST_PATH = 'lingva.ml';

const LEGACY_LINGVA_REQUEST_PATH = 'lingva.pot-app.com';
const FALLBACK_LINGVA_REQUEST_PATHS = [
    DEFAULT_LINGVA_REQUEST_PATH,
    'translate.plausibility.cloud',
    'lingva.lunar.icu',
];

function normaliseRequestPath(requestPath) {
    return requestPath
        .trim()
        .replace(/^https?:\/\//i, '')
        .replace(/\/+$/, '')
        .toLowerCase();
}

function toOrigin(requestPath) {
    return /^https?:\/\//i.test(requestPath) ? requestPath.replace(/\/+$/, '') : `https://${requestPath}`;
}

export function getLingvaRequestPaths(requestPath) {
    const configuredPath = typeof requestPath === 'string' ? requestPath.trim() : '';
    const normalisedPath = normaliseRequestPath(configuredPath);
    if (normalisedPath === '' || normalisedPath === LEGACY_LINGVA_REQUEST_PATH) {
        return FALLBACK_LINGVA_REQUEST_PATHS.map(toOrigin);
    }
    return [toOrigin(configuredPath)];
}
