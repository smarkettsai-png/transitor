import { exists } from '@tauri-apps/api/fs';
import { invoke } from '@tauri-apps/api/tauri';

import { appConfigDirPath, appDataDirPath } from '../../../utils/paths';

async function rustPortableDatabasePath() {
    try {
        const databasePath = await invoke('ecdict_database_path');
        if (typeof databasePath === 'string' && databasePath.length > 0) {
            return databasePath;
        }
    } catch {
        // Older builds do not expose the command; use the frontend fallbacks.
    }
    return null;
}

export async function ecdictDatabasePath() {
    const candidates = [];
    const rustPath = await rustPortableDatabasePath();
    if (rustPath) {
        candidates.push(rustPath);
    }

    try {
        candidates.push(await appDataDirPath('ecdict/stardict.db'));
    } catch {
        // Continue with the plugin location below.
    }
    try {
        candidates.push(await appConfigDirPath('plugins/translate/plugin.com.pot-app.ecdict/stardict.db'));
    } catch {
        // No configured path is available.
    }

    for (const candidate of [...new Set(candidates)]) {
        try {
            if (await exists(candidate)) {
                return candidate;
            }
        } catch {
            // An out-of-scope candidate should not prevent the next location from being checked.
        }
    }
    return null;
}

export async function ecdictAvailable() {
    return (await ecdictDatabasePath()) !== null;
}
