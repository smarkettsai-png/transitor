use crate::config::get;
use crate::config::StoreWrapper;
use crate::error::Error;
use crate::portable;
use crate::StringWrapper;
use crate::APP;
use log::{error, info};
use serde_json::{json, Value};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions},
    Row,
};
use std::io::Read;
use tauri::Manager;

#[tauri::command]
pub fn get_text(state: tauri::State<StringWrapper>) -> String {
    return state.0.lock().unwrap().to_string();
}

#[tauri::command]
pub fn reload_store() {
    let state = APP.get().unwrap().state::<StoreWrapper>();
    let mut store = state.0.lock().unwrap();
    store.load().unwrap();
}

#[tauri::command]
pub fn cut_image(left: u32, top: u32, width: u32, height: u32, app_handle: tauri::AppHandle) {
    use image::GenericImage;
    info!("Cut image: {}x{}+{}+{}", width, height, left, top);
    let mut app_cache_dir_path = portable::cache_dir(&app_handle.config().tauri.bundle.identifier)
        .expect("Get Cache Dir Failed");
    app_cache_dir_path.push("pot_screenshot.png");
    if !app_cache_dir_path.exists() {
        return;
    }
    let mut img = match image::open(&app_cache_dir_path) {
        Ok(v) => v,
        Err(e) => {
            error!("{:?}", e.to_string());
            return;
        }
    };
    let img2 = img.sub_image(left, top, width, height);
    app_cache_dir_path.pop();
    app_cache_dir_path.push("pot_screenshot_cut.png");
    match img2.to_image().save(&app_cache_dir_path) {
        Ok(_) => {}
        Err(e) => {
            error!("{:?}", e.to_string());
        }
    }
}

#[tauri::command]
pub fn get_base64(app_handle: tauri::AppHandle) -> String {
    use base64::{engine::general_purpose, Engine as _};
    use std::fs::File;
    use std::io::Read;
    let mut app_cache_dir_path = portable::cache_dir(&app_handle.config().tauri.bundle.identifier)
        .expect("Get Cache Dir Failed");
    app_cache_dir_path.push("pot_screenshot_cut.png");
    if !app_cache_dir_path.exists() {
        return "".to_string();
    }
    let mut file = File::open(app_cache_dir_path).unwrap();
    let mut vec = Vec::new();
    match file.read_to_end(&mut vec) {
        Ok(_) => {}
        Err(e) => {
            error!("{:?}", e.to_string());
            return "".to_string();
        }
    }
    let base64 = general_purpose::STANDARD.encode(&vec);
    base64.replace("\r\n", "")
}

#[tauri::command]
pub fn copy_img(app_handle: tauri::AppHandle, width: usize, height: usize) -> Result<(), Error> {
    use arboard::{Clipboard, ImageData};
    use image::ImageReader;
    use std::borrow::Cow;

    let mut app_cache_dir_path = portable::cache_dir(&app_handle.config().tauri.bundle.identifier)
        .expect("Get Cache Dir Failed");
    app_cache_dir_path.push("pot_screenshot_cut.png");
    let data = ImageReader::open(app_cache_dir_path)?.decode()?;

    let img = ImageData {
        width,
        height,
        bytes: Cow::from(data.as_bytes()),
    };
    let result = Clipboard::new()?.set_image(img)?;
    Ok(result)
}

#[tauri::command]
pub fn set_proxy() -> Result<bool, ()> {
    let host = match get("proxy_host") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => return Err(()),
    };
    let port = match get("proxy_port") {
        Some(v) => v.as_i64().unwrap(),
        None => return Err(()),
    };
    let no_proxy = match get("no_proxy") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => return Err(()),
    };
    let proxy = format!("http://{}:{}", host, port);

    std::env::set_var("http_proxy", &proxy);
    std::env::set_var("https_proxy", &proxy);
    std::env::set_var("all_proxy", &proxy);
    std::env::set_var("no_proxy", &no_proxy);
    Ok(true)
}

#[tauri::command]
pub fn unset_proxy() -> Result<bool, ()> {
    std::env::remove_var("http_proxy");
    std::env::remove_var("https_proxy");
    std::env::remove_var("all_proxy");
    std::env::remove_var("no_proxy");
    Ok(true)
}

#[tauri::command]
pub fn install_plugin(path_list: Vec<String>) -> Result<i32, Error> {
    let mut success_count = 0;

    for path in path_list {
        if !path.ends_with("potext") {
            continue;
        }
        let path = std::path::Path::new(&path);
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let file_name = file_name.replace(".potext", "");
        if !file_name.starts_with("plugin") {
            return Err(Error::Error(
                "Invalid Plugin: file name must start with plugin".into(),
            ));
        }

        let mut zip = zip::ZipArchive::new(std::fs::File::open(path)?)?;
        #[allow(unused_mut)]
        let mut plugin_type: String;
        if let Ok(mut info) = zip.by_name("info.json") {
            let mut content = String::new();
            info.read_to_string(&mut content)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;
            plugin_type = json["plugin_type"]
                .as_str()
                .ok_or(Error::Error("can't find plugin type in info.json".into()))?
                .to_string();
        } else {
            return Err(Error::Error("Invalid Plugin: miss info.json".into()));
        }
        if zip.by_name("main.js").is_err() {
            return Err(Error::Error("Invalid Plugin: miss main.js".into()));
        }
        let config_path =
            portable::config_dir(&APP.get().unwrap().config().tauri.bundle.identifier)
                .expect("Get Config Dir Failed");
        let config_path = config_path.join("plugins");
        let config_path = config_path.join(plugin_type);
        let plugin_path = config_path.join(file_name);
        std::fs::create_dir_all(&config_path)?;
        zip.extract(&plugin_path)?;

        success_count += 1;
    }
    Ok(success_count)
}

#[derive(Debug, serde::Serialize)]
pub struct EcdictEntry {
    pub word: String,
    pub phonetic: Option<String>,
    pub definition: Option<String>,
    pub translation: Option<String>,
    pub tag: Option<String>,
    pub exchange: Option<String>,
}

async fn query_ecdict_pool(pool: &SqlitePool, word: &str) -> Result<Option<EcdictEntry>, Error> {
    let row = sqlx::query(
        "SELECT word, phonetic, definition, translation, tag, exchange FROM stardict WHERE word = ? LIMIT 1",
    )
    .bind(word)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        Ok(Some(EcdictEntry {
            word: row.try_get("word")?,
            phonetic: row.try_get("phonetic")?,
            definition: row.try_get("definition")?,
            translation: row.try_get("translation")?,
            tag: row.try_get("tag")?,
            exchange: row.try_get("exchange")?,
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn ecdict_lookup(
    word: String,
    app_handle: tauri::AppHandle,
) -> Result<Option<EcdictEntry>, Error> {
    let word = word.trim();
    if word.is_empty() {
        return Ok(None);
    }

    let database_path = portable::ecdict_database_file(
        &app_handle.config().tauri.bundle.identifier,
    )
    .ok_or_else(|| {
        Error::Error(
            "ECDict database not found. Put stardict.db under data/ecdict or install the ECDict plugin."
                .into(),
        )
    })?;
    let options = SqliteConnectOptions::new()
        .filename(database_path)
        .read_only(true)
        .create_if_missing(false);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    let entry = query_ecdict_pool(&pool, word).await?;
    pool.close().await;
    Ok(entry)
}

#[derive(Debug, serde::Serialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub text: String,
    pub source: String,
    pub target: String,
    pub service: String,
    pub result: String,
    pub timestamp: i64,
}

const HISTORY_TABLE: &str = "CREATE TABLE IF NOT EXISTS history(\
    id INTEGER PRIMARY KEY AUTOINCREMENT,\
    text TEXT NOT NULL,\
    source TEXT NOT NULL,\
    target TEXT NOT NULL,\
    service TEXT NOT NULL,\
    result TEXT NOT NULL,\
    timestamp INTEGER NOT NULL\
)";

async fn history_pool(app_handle: &tauri::AppHandle) -> Result<SqlitePool, Error> {
    let config_dir = portable::config_dir(&app_handle.config().tauri.bundle.identifier)
        .ok_or_else(|| Error::Error("History config path unavailable".into()))?;
    std::fs::create_dir_all(&config_dir)?;
    let options = SqliteConnectOptions::new()
        .filename(config_dir.join("history.db"))
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    sqlx::query(HISTORY_TABLE).execute(&pool).await?;
    Ok(pool)
}

fn history_entry_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<HistoryEntry, Error> {
    Ok(HistoryEntry {
        id: row.try_get("id")?,
        text: row.try_get("text")?,
        source: row.try_get("source")?,
        target: row.try_get("target")?,
        service: row.try_get("service")?,
        result: row.try_get("result")?,
        timestamp: row.try_get("timestamp")?,
    })
}

#[tauri::command]
pub async fn history_add(
    app_handle: tauri::AppHandle,
    text: String,
    source: String,
    target: String,
    service: String,
    result: String,
    timestamp: i64,
) -> Result<(), Error> {
    let pool = history_pool(&app_handle).await?;
    sqlx::query(
        "INSERT INTO history (text, source, target, service, result, timestamp) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(text)
    .bind(source)
    .bind(target)
    .bind(service)
    .bind(result)
    .bind(timestamp)
    .execute(&pool)
    .await?;
    pool.close().await;
    Ok(())
}

#[tauri::command]
pub async fn history_count(app_handle: tauri::AppHandle) -> Result<i64, Error> {
    let pool = history_pool(&app_handle).await?;
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM history")
        .fetch_one(&pool)
        .await?;
    pool.close().await;
    Ok(count)
}

#[tauri::command]
pub async fn history_list(
    app_handle: tauri::AppHandle,
    page: i64,
) -> Result<Vec<HistoryEntry>, Error> {
    let pool = history_pool(&app_handle).await?;
    let offset = page.max(1).saturating_sub(1) * 20;
    let rows = sqlx::query(
        "SELECT id, text, source, target, service, result, timestamp FROM history ORDER BY id DESC LIMIT 20 OFFSET ?",
    )
    .bind(offset)
    .fetch_all(&pool)
    .await?;
    let entries = rows
        .iter()
        .map(history_entry_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    pool.close().await;
    Ok(entries)
}

#[tauri::command]
pub async fn history_get(
    app_handle: tauri::AppHandle,
    id: i64,
) -> Result<Option<HistoryEntry>, Error> {
    let pool = history_pool(&app_handle).await?;
    let row = sqlx::query(
        "SELECT id, text, source, target, service, result, timestamp FROM history WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?;
    let entry = row.as_ref().map(history_entry_from_row).transpose()?;
    pool.close().await;
    Ok(entry)
}

#[tauri::command]
pub async fn history_clear(app_handle: tauri::AppHandle) -> Result<(), Error> {
    let pool = history_pool(&app_handle).await?;
    sqlx::query("DELETE FROM history").execute(&pool).await?;
    pool.close().await;
    Ok(())
}

#[tauri::command]
pub async fn history_update(
    app_handle: tauri::AppHandle,
    id: i64,
    text: String,
    result: String,
) -> Result<(), Error> {
    let pool = history_pool(&app_handle).await?;
    sqlx::query("UPDATE history SET text = ?, result = ? WHERE id = ?")
        .bind(text)
        .bind(result)
        .bind(id)
        .execute(&pool)
        .await?;
    pool.close().await;
    Ok(())
}

#[tauri::command]
pub fn run_binary(
    plugin_type: String,
    plugin_name: String,
    cmd_name: String,
    args: Vec<String>,
) -> Result<Value, Error> {
    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    let config_path = portable::config_dir(&APP.get().unwrap().config().tauri.bundle.identifier)
        .expect("Get Config Dir Failed");
    let config_path = config_path.join("plugins");
    let config_path = config_path.join(plugin_type);
    let plugin_path = config_path.join(plugin_name);

    #[cfg(target_os = "windows")]
    let mut cmd = Command::new("cmd");
    #[cfg(target_os = "windows")]
    let cmd = cmd.creation_flags(0x08000000);
    #[cfg(target_os = "windows")]
    let cmd = cmd.args(["/c", &cmd_name]);
    #[cfg(not(target_os = "windows"))]
    let mut cmd = Command::new(&cmd_name);

    let output = cmd.args(args).current_dir(plugin_path).output()?;
    Ok(json!({
        "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
        "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
        "status": output.status.code().unwrap_or(-1),
    }))
}

#[tauri::command]
pub fn font_list() -> Result<Vec<String>, Error> {
    use font_kit::source::SystemSource;
    let source = SystemSource::new();

    Ok(source.all_families()?)
}

#[tauri::command]
pub fn open_devtools(window: tauri::Window) {
    if !window.is_devtools_open() {
        window.open_devtools();
    } else {
        window.close_devtools();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ecdict_query_returns_dictionary_fields() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE stardict (word TEXT PRIMARY KEY, phonetic TEXT, definition TEXT, translation TEXT, tag TEXT, exchange TEXT)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO stardict (word, phonetic, definition, translation, tag, exchange) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind("hello")
        .bind("həˈləʊ")
        .bind("int. greeting")
        .bind("你好")
        .bind("common")
        .bind("0:hello")
        .execute(&pool)
        .await
        .unwrap();

        let result = query_ecdict_pool(&pool, "hello").await.unwrap().unwrap();
        assert_eq!(result.word, "hello");
        assert_eq!(result.phonetic.as_deref(), Some("həˈləʊ"));
        assert_eq!(result.translation.as_deref(), Some("你好"));
    }
}
