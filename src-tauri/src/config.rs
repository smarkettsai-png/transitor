use crate::portable;
use crate::{error::Error, APP};
use log::{info, warn};
use serde_json::{json, Value};
use std::sync::Mutex;
use tauri::{Manager, Wry};
use tauri_plugin_store::{Store, StoreBuilder};

pub struct StoreWrapper(pub Mutex<Store<Wry>>);

pub fn init_config(app: &mut tauri::App) -> bool {
    let config_path = portable::config_dir(&app.config().tauri.bundle.identifier)
        .expect("Get Config Dir Failed")
        .join("config.json");
    info!("Load config from: {:?}", config_path);
    let mut store = StoreBuilder::new(app.handle(), config_path).build();

    match store.load() {
        Ok(_) => info!("Config loaded"),
        Err(e) => {
            warn!("Config load error: {:?}", e);
            info!("Config not found, creating new config");
        }
    }
    let first_run = store.is_empty();
    app.manage(StoreWrapper(Mutex::new(store)));
    migrate_language_detection_setting();
    let _ = check_service_available();
    ensure_local_translate_services(&app.config().tauri.bundle.identifier);
    first_run
}

fn migrate_language_detection_setting() {
    if let Some(Value::String(engine)) = get("translate_detect_engine") {
        if engine == "baidu" {
            info!("Migrating Baidu language detection to local detection");
            set("translate_detect_engine", "local");
        }
    }
}

fn check_available(list: Vec<String>, builtin: Vec<&str>, plugin: Vec<String>, key: &str) {
    let origin_length = list.len();
    let mut new_list = list.clone();
    for service in list {
        let name = service.split("@").collect::<Vec<&str>>()[0];
        let mut is_available = true;
        // ECDict is now a built-in native service. Do not keep an old
        // ECDict plugin instance around, because that plugin still calls the
        // SQL plugin with an absolute Windows path and triggers the malformed
        // connection URL regression.
        if name == "plugin.com.pot-app.ecdict" {
            is_available = false;
        } else if name.starts_with("plugin") {
            if !plugin.contains(&name.to_string()) {
                is_available = false;
            }
        } else {
            if !builtin.contains(&name) {
                is_available = false;
            }
        }
        if !is_available {
            new_list.retain(|x| x != &service);
        }
    }
    if new_list.len() != origin_length {
        set(key, new_list);
    }
}

pub fn ensure_local_translate_services(identifier: &str) {
    let mut changed = false;
    let mut services = get("translate_service_list")
        .and_then(|value| serde_json::from_value::<Vec<String>>(value).ok())
        .unwrap_or_else(|| {
            changed = true;
            vec![
                "deepl".to_string(),
                "bing".to_string(),
                "lingva".to_string(),
                "yandex".to_string(),
                "google".to_string(),
                "mymemory".to_string(),
            ]
        });
    if portable::ecdict_database_file(identifier).is_some()
        && !services.iter().any(|x| x == "ecdict")
    {
        services.insert(0, "ecdict".to_string());
        changed = true;
    }
    let tatoeba_info = portable::config_dir(identifier)
        .map(|path| {
            path.join("plugins")
                .join("translate")
                .join("plugin.com.pot-app.tatoeba")
                .join("info.json")
        })
        .is_some_and(|path| path.is_file());
    if tatoeba_info && !services.iter().any(|x| x == "plugin.com.pot-app.tatoeba") {
        services.push("plugin.com.pot-app.tatoeba".to_string());
        changed = true;
    }
    if changed {
        set("translate_service_list", services);
    }
}

pub fn check_service_available() -> Result<(), Error> {
    let builtin_recognize_list: Vec<&str> = vec![
        "baidu_ocr",
        "baidu_accurate_ocr",
        "baidu_img_ocr",
        "iflytek_ocr",
        "iflytek_intsig_ocr",
        "iflytek_latex_ocr",
        "qrcode",
        "simple_latex_ocr",
        "system",
        "tencent_ocr",
        "tencent_accurate_ocr",
        "tencent_img_ocr",
        "tesseract",
        "volcengine_ocr",
        "volcengine_multi_lang_ocr",
    ];
    let builtin_translate_list: Vec<&str> = vec![
        "alibaba",
        "baidu",
        "baidu_field",
        "bing",
        "bing_dict",
        "caiyun",
        "cambridge_dict",
        "chatglm",
        "deepl",
        "ecdict",
        "lingva",
        "mymemory",
        "geminipro",
        "niutrans",
        "ollama",
        "openai",
        "google",
        "tencent",
        "transmart",
        "volcengine",
        "yandex",
        "youdao",
    ];
    let builtin_tts_list: Vec<&str> = vec!["lingva_tts"];
    let builtin_collection_list: Vec<&str> = vec!["anki", "eudic"];

    let plugin_recognize_list: Vec<String> = get_plugin_list("recognize").unwrap_or_default();
    let plugin_translate_list: Vec<String> = get_plugin_list("translate").unwrap_or_default();
    let plugin_tts_list: Vec<String> = get_plugin_list("tts").unwrap_or_default();
    let plugin_collection_list: Vec<String> = get_plugin_list("collection").unwrap_or_default();
    if let Some(recognize_service_list) = get("recognize_service_list") {
        let recognize_service_list: Vec<String> = serde_json::from_value(recognize_service_list)?;
        check_available(
            recognize_service_list,
            builtin_recognize_list,
            plugin_recognize_list,
            "recognize_service_list",
        );
    }
    if let Some(translate_service_list) = get("translate_service_list") {
        let translate_service_list: Vec<String> = serde_json::from_value(translate_service_list)?;
        check_available(
            translate_service_list,
            builtin_translate_list,
            plugin_translate_list,
            "translate_service_list",
        );
    }
    if let Some(tts_service_list) = get("tts_service_list") {
        let tts_service_list: Vec<String> = serde_json::from_value(tts_service_list)?;
        check_available(
            tts_service_list,
            builtin_tts_list,
            plugin_tts_list,
            "tts_service_list",
        );
    }
    if let Some(collection_service_list) = get("collection_service_list") {
        let collection_service_list: Vec<String> = serde_json::from_value(collection_service_list)?;
        check_available(
            collection_service_list,
            builtin_collection_list,
            plugin_collection_list,
            "collection_service_list",
        );
    }
    Ok(())
}

pub fn get_plugin_list(plugin_type: &str) -> Option<Vec<String>> {
    let app_handle = APP.get().unwrap();
    let config_dir = portable::config_dir(&app_handle.config().tauri.bundle.identifier)?;
    let plugin_dir = config_dir.join("plugins");
    let plugin_dir = plugin_dir.join(plugin_type);

    // dirs in plugin_dir
    let mut plugin_list = vec![];
    if plugin_dir.exists() {
        let read_dir = std::fs::read_dir(plugin_dir).ok()?;
        for entry in read_dir {
            let entry = entry.ok()?;

            if entry.path().is_dir() {
                let name = entry.file_name().to_str()?.to_string();
                if name.starts_with("plugin") {
                    plugin_list.push(name);
                } else {
                    // Remove old plugin
                    let _ = std::fs::remove_dir_all(entry.path());
                }
            }
        }
    }
    Some(plugin_list)
}

pub fn get(key: &str) -> Option<Value> {
    let state = APP.get().unwrap().state::<StoreWrapper>();
    let store = state.0.lock().unwrap();
    match store.get(key) {
        Some(value) => Some(value.clone()),
        None => None,
    }
}

pub fn set<T: serde::ser::Serialize>(key: &str, value: T) {
    let state = APP.get().unwrap().state::<StoreWrapper>();
    let mut store = state.0.lock().unwrap();
    store.insert(key.to_string(), json!(value)).unwrap();
    store.save().unwrap();
}
