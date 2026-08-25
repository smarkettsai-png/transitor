import { invoke } from '@tauri-apps/api';
import { join } from '@tauri-apps/api/path';

let pathsPromise;

async function paths() {
    pathsPromise ??= invoke('portable_paths');
    return pathsPromise;
}

export async function appConfigDirPath(relative = '') {
    const { configDir } = await paths();
    return relative ? join(configDir, relative) : configDir;
}

export async function appCacheDirPath(relative = '') {
    const { cacheDir } = await paths();
    return relative ? join(cacheDir, relative) : cacheDir;
}

export async function appDataDirPath(relative = '') {
    const { dataDir } = await paths();
    return relative ? join(dataDir, relative) : dataDir;
}

export const configPath = appConfigDirPath;
export const cachePath = appCacheDirPath;
