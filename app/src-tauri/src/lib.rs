//! Backend do app: comandos Tauri sobre `salamandra_core::Mouse` e o serviço
//! de atalhos rodando numa thread, que repassa os avisos do mouse para a tela.

use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use salamandra_core::defaults::factory_profile;
use salamandra_core::{map, Error, MacroEvent, Mouse, Profile};
use salamandra_hid::HidTransport;
use serde::Serialize;
use tauri::{Emitter, Manager, State};

#[derive(Default)]
struct AppState {
    mouse: Mutex<Option<Mouse<HidTransport>>>,
    /// Perfis como estão no mouse, para gravar só o que mudou.
    loaded: Mutex<Vec<Profile>>,
}

#[derive(Serialize)]
struct Snapshot {
    active_profile: usize,
    profiles: Vec<Profile>,
    macros: Vec<Vec<MacroEvent>>,
}

/// Roda `f` com o mouse aberto (abre se preciso). Em falha de comunicação,
/// descarta a conexão para a próxima chamada reabrir.
fn with_mouse<R>(
    state: &AppState,
    f: impl FnOnce(&mut Mouse<HidTransport>) -> Result<R, Error>,
) -> Result<R, String> {
    let mut guard = state.mouse.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        *guard = Some(salamandra_hid::open().map_err(|e| e.to_string())?);
    }
    let result = f(guard.as_mut().expect("aberto acima"));
    if let Err(Error::Io(_)) = &result {
        *guard = None;
    }
    result.map_err(|e| e.to_string())
}

fn read_all(mouse: &mut Mouse<HidTransport>) -> Result<Snapshot, Error> {
    Ok(Snapshot {
        active_profile: mouse.active_profile()?,
        profiles: (0..map::PROFILE_COUNT)
            .map(|p| mouse.profile(p))
            .collect::<Result<_, _>>()?,
        macros: (0..map::MACRO_COUNT)
            .map(|i| mouse.macro_events(i))
            .collect::<Result<_, _>>()?,
    })
}

fn remember(state: &AppState, snapshot: &Snapshot) {
    if let Ok(mut loaded) = state.loaded.lock() {
        *loaded = snapshot.profiles.clone();
    }
}

#[tauri::command]
fn connect(state: State<AppState>) -> Result<Snapshot, String> {
    let snapshot = with_mouse(&state, read_all)?;
    remember(&state, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
fn save_profile(state: State<AppState>, index: usize, profile: Profile) -> Result<Profile, String> {
    let old = state
        .loaded
        .lock()
        .map_err(|e| e.to_string())?
        .get(index)
        .cloned()
        .ok_or("perfil inválido")?;
    let fresh = with_mouse(&state, |m| {
        m.update_profile(index, &old, &profile)?;
        m.profile(index)
    })?;
    state.loaded.lock().map_err(|e| e.to_string())?[index] = fresh.clone();
    Ok(fresh)
}

#[tauri::command]
fn set_active_profile(state: State<AppState>, index: usize) -> Result<usize, String> {
    with_mouse(&state, |m| {
        m.set_active_profile(index)?;
        m.active_profile()
    })
}

#[tauri::command]
fn set_macro(
    state: State<AppState>,
    index: usize,
    events: Vec<MacroEvent>,
) -> Result<Vec<MacroEvent>, String> {
    with_mouse(&state, |m| {
        m.set_macro(index, &events)?;
        m.macro_events(index)
    })
}

#[tauri::command]
fn factory_defaults(index: usize) -> Result<Profile, String> {
    if index >= map::PROFILE_COUNT {
        return Err("perfil inválido".into());
    }
    Ok(factory_profile(index))
}

#[tauri::command]
fn backup(state: State<AppState>, path: String) -> Result<(), String> {
    let data = with_mouse(&state, |m| m.backup())?;
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
fn restore(state: State<AppState>, path: String) -> Result<Snapshot, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let data: BTreeMap<String, String> =
        serde_json::from_str(&text).map_err(|_| "arquivo de backup inválido")?;
    let snapshot = with_mouse(&state, |m| {
        m.restore(&data)?;
        read_all(m)
    })?;
    remember(&state, &snapshot);
    Ok(snapshot)
}

fn macro_names_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("macro-names.json"))
}

/// Nomes das macros: como no oficial, ficam só no computador.
#[tauri::command]
fn macro_names(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let path = macro_names_path(&app)?;
    let mut names: Vec<String> = std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    names.resize(map::MACRO_COUNT, String::new());
    Ok(names)
}

#[tauri::command]
fn set_macro_names(app: tauri::AppHandle, names: Vec<String>) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&names).map_err(|e| e.to_string())?;
    std::fs::write(macro_names_path(&app)?, json).map_err(|e| e.to_string())
}

/// Nomes das teclas (uso HID), a mesma tabela do CLI.
#[tauri::command]
fn key_names() -> Vec<(String, u8)> {
    salamandra_core::keys::all().to_vec()
}

static STOP_SERVICE: AtomicBool = AtomicBool::new(false);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let errors = handle.clone();
                salamandra_service::run(
                    &STOP_SERVICE,
                    |event| {
                        let _ = handle.emit("device-event", event);
                    },
                    |error| {
                        let _ = errors.emit("service-error", error.to_string());
                    },
                );
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connect,
            save_profile,
            set_active_profile,
            set_macro,
            factory_defaults,
            key_names,
            macro_names,
            set_macro_names,
            backup,
            restore
        ])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o app");
}
