import { readDir, readTextFile, exists } from '@tauri-apps/api/fs';
import { listen } from '@tauri-apps/api/event';
import { useTranslation } from 'react-i18next';
import { Tabs, Tab } from '@nextui-org/react';
import { appConfigDirPath, configPath } from '../../../../utils/paths';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import React, { useEffect, useState } from 'react';
import Translate from './Translate';
import Recognize from './Recognize';
import Collection from './Collection';
import Tts from './Tts';
import { ServiceType } from '../../../../utils/service_instance';
import { info } from 'tauri-plugin-log-api';

let unlisten = null;

export default function Service() {
    const [pluginList, setPluginList] = useState(null);
    const { t } = useTranslation();

    const loadPluginList = async () => {
        const serviceTypeList = ['translate', 'tts', 'recognize', 'collection'];
        const temp = {};
        for (const serviceType of serviceTypeList) {
            temp[serviceType] = {};
            try {
                const pluginDirectory = await configPath(`plugins/${serviceType}`);
                if (!(await exists(pluginDirectory))) {
                    continue;
                }
                const plugins = await readDir(pluginDirectory);
                for (const plugin of plugins) {
                    const pluginName = plugin.name ?? plugin.path?.split(/[\\/]/).pop();
                    if (!pluginName) continue;
                    try {
                        const infoStr = await readTextFile(
                            await configPath(`plugins/${serviceType}/${pluginName}/info.json`)
                        );
                        const pluginInfo = JSON.parse(infoStr);
                        if ('icon' in pluginInfo) {
                            const iconPath = await appConfigDirPath(
                                `plugins/${serviceType}/${pluginName}/${pluginInfo.icon}`
                            );
                            pluginInfo.icon = convertFileSrc(iconPath);
                        }
                        temp[serviceType][pluginName] = pluginInfo;
                    } catch (error) {
                        info(`[plugin] skip invalid ${serviceType}/${pluginName}: ${error}`);
                    }
                }
            } catch (error) {
                info(`[plugin] cannot load ${serviceType} plugins: ${error}`);
            }
        }
        setPluginList(temp);
    };

    useEffect(() => {
        loadPluginList();
        if (unlisten) {
            unlisten.then((f) => {
                f();
            });
        }
        unlisten = listen('reload_plugin_list', loadPluginList);
        return () => {
            if (unlisten) {
                unlisten.then((f) => {
                    f();
                });
            }
        };
    }, []);
    return (
        pluginList !== null && (
            <Tabs className='flex justify-center max-h-[calc(100%-40px)] overflow-y-auto'>
                <Tab
                    key='translate'
                    title={t(`config.service.translate`)}
                >
                    <Translate pluginList={pluginList[ServiceType.TRANSLATE]} />
                </Tab>
                <Tab
                    key='recognize'
                    title={t(`config.service.recognize`)}
                >
                    <Recognize pluginList={pluginList[ServiceType.RECOGNIZE]} />
                </Tab>
                <Tab
                    key='tts'
                    title={t(`config.service.tts`)}
                >
                    <Tts pluginList={pluginList[ServiceType.TTS]} />
                </Tab>
                <Tab
                    key='collection'
                    title={t(`config.service.collection`)}
                >
                    <Collection pluginList={pluginList[ServiceType.COLLECTION]} />
                </Tab>
            </Tabs>
        )
    );
}
