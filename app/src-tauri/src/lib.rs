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

const PROFILE_FORMAT: &str = "salamandra-profile";

/// Arquivo de um perfil: formato próprio em JSON, com versão.
#[derive(Serialize, serde::Deserialize)]
struct ProfileFile {
    format: String,
    version: u32,
    profile: Profile,
}

#[tauri::command]
fn export_profile(path: String, profile: Profile) -> Result<(), String> {
    let file = ProfileFile {
        format: PROFILE_FORMAT.into(),
        version: 1,
        profile,
    };
    let json = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_profile(path: String) -> Result<Profile, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let file: ProfileFile =
        serde_json::from_str(&text).map_err(|_| "arquivo de perfil inválido")?;
    if file.format != PROFILE_FORMAT || file.version != 1 {
        return Err("arquivo de perfil de outro formato ou versão".into());
    }
    if file.profile.buttons.len() != map::BUTTON_COUNT {
        return Err("arquivo de perfil com número errado de botões".into());
    }
    file.profile.validate().map_err(|e| e.to_string())?;
    Ok(file.profile)
}

/// "Reset all" do oficial: os 5 perfis voltam para o padrão de fábrica.
#[tauri::command]
fn reset_all(state: State<AppState>) -> Result<Snapshot, String> {
    let snapshot = with_mouse(&state, |m| {
        for p in 0..map::PROFILE_COUNT {
            let old = m.profile(p)?;
            m.update_profile(p, &old, &factory_profile(p))?;
        }
        read_all(m)
    })?;
    remember(&state, &snapshot);
    Ok(snapshot)
}

static STOP_SERVICE: AtomicBool = AtomicBool::new(false);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
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
            export_profile,
            import_profile,
            reset_all,
            backup,
            restore
        ])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o app");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_file_round_trip() {
        let path = std::env::temp_dir().join("salamandra-perfil-teste.json");
        let path = path.to_string_lossy().to_string();
        let profile = factory_profile(2);
        export_profile(path.clone(), profile.clone()).unwrap();
        assert_eq!(import_profile(path.clone()).unwrap(), profile);

        let mut broken = profile.clone();
        broken.buttons[0] = salamandra_core::ButtonAction::Disabled;
        export_profile(path.clone(), broken).unwrap();
        assert!(
            import_profile(path.clone()).is_err(),
            "sem clique esquerdo não entra"
        );

        std::fs::write(&path, "{\"format\":\"outro\"}").unwrap();
        assert!(import_profile(path.clone()).is_err());
        let _ = std::fs::remove_file(path);
    }
}
