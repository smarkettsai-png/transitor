import { Store } from 'tauri-plugin-store-api';
import { appConfigDirPath } from './paths';
import { watch } from 'tauri-plugin-fs-watch-api';
import { invoke } from '@tauri-apps/api';

export let store = new Store();

export async function initStore() {
    const appConfigPath = await appConfigDirPath('config.json');
    store = new Store(appConfigPath);
    const _ = await watch(appConfigPath, async () => {
        await store.load();
        await invoke('reload_store');
    });
}
