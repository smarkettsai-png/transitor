import { appCacheDirPath, appConfigDirPath } from './paths';
import { readBinaryFile, readTextFile } from '@tauri-apps/api/fs';
import Database from 'tauri-plugin-sql-api';
import { http } from '@tauri-apps/api';
import CryptoJS from 'crypto-js';
import { osType } from './env';

export async function invoke_plugin(pluginType, pluginName) {
    let configDir = await appConfigDirPath();
    let cacheDir = await appCacheDirPath();
    let pluginDir = await appConfigDirPath(['plugins', pluginType, pluginName].join('/'));
    let entryFile = await appConfigDirPath(['plugins', pluginType, pluginName, 'main.js'].join('/'));
    let script = await readTextFile(entryFile);
    const utils = {
        tauriFetch: http.fetch,
        http,
        readBinaryFile,
        readTextFile,
        Database,
        CryptoJS,
        cacheDir, // String
        pluginDir, // String
        osType, // "Windows_NT", "Darwin", "Linux"
    };
    return [eval(`${script} ${pluginType}`), utils];
}
