export const TATOEBA_PLUGIN_ID = 'plugin.com.pot-app.tatoeba';

export function addAvailableTranslateServices(serviceList, { tatoeba = false, ecdict = false } = {}) {
    const next = [...serviceList];
    if (ecdict && !next.includes('ecdict')) {
        next.unshift('ecdict');
    }
    if (tatoeba && !next.includes(TATOEBA_PLUGIN_ID)) {
        next.push(TATOEBA_PLUGIN_ID);
    }
    return next;
}
