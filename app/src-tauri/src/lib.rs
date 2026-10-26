//! Backend do app: comandos Tauri sobre `salamandra_core::Mouse` e o serviço
//! de atalhos rodando numa thread, que repassa os avisos do mouse para a tela.

use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use salamandra_core::defaults::factory_profile_for;
use salamandra_core::{map, Error, MacroEvent, Mouse, Profile, Transport};
use salamandra_hid::HidTransport;
use serde::Serialize;
use tauri::{Emitter, Manager, State};

#[derive(Default)]
struct AppState {
    mouse: Mutex<Option<Mouse<HidTransport>>>,
    /// Perfis como estão no mouse, para gravar só o que mudou.
    loaded: Mutex<Vec<Profile>>,
    /// Id do modelo que o usuário escolheu quando há mais de um mouse conectado.
    preferred: Mutex<Option<String>>,
    /// Itens do menu da bandeja, para o front trocar o texto com o idioma.
    tray_items: Mutex<
        Option<(
            tauri::menu::MenuItem<tauri::Wry>,
            tauri::menu::MenuItem<tauri::Wry>,
        )>,
    >,
}

/// O modelo conectado, para a tela adaptar DPI, botões e o aviso de "não testado".
#[derive(Serialize)]
struct ModelInfo {
    id: &'static str,
    name: &'static str,
    vendor_id: u16,
    product_id: u16,
    /// Versão do dispositivo (`bcdDevice`); o mesmo PID cobre modelos diferentes.
    release: Option<u16>,
    tested: bool,
    source: &'static str,
    buttons: Vec<usize>,
    dpi_values: Vec<u32>,
    macro_slots: usize,
    /// Modos de LED que o modelo oferece, como os nomes de `LedMode`.
    led_modes: Vec<&'static str>,
}

impl ModelInfo {
    fn of(spec: &salamandra_core::DeviceSpec, release: Option<u16>) -> Self {
        Self {
            id: spec.id,
            name: spec.name,
            vendor_id: spec.vendor_id,
            product_id: spec.product_id,
            release,
            tested: spec.is_tested(),
            source: spec.source,
            buttons: spec.buttons.to_vec(),
            dpi_values: spec.dpi.values(),
            macro_slots: spec.macro_slots,
            led_modes: spec.led.modes().to_vec(),
        }
    }
}

#[derive(Serialize)]
struct Snapshot {
    model: ModelInfo,
    active_profile: usize,
    profiles: Vec<Profile>,
    macros: Vec<Vec<MacroEvent>>,
}

fn is_plugged(id: &str, connected: &[&str]) -> bool {
    connected.iter().any(|c| c.eq_ignore_ascii_case(id))
}

/// Roda `f` com o mouse aberto (abre se preciso). Reabre sempre o modelo escolhido
/// e nunca troca de mouse sozinho: quem troca é `connect`, que relê os perfis. Em falha de comunicação,
/// descarta a conexão para a próxima chamada reabrir.
fn with_mouse<R>(
    state: &AppState,
    f: impl FnOnce(&mut Mouse<HidTransport>) -> Result<R, Error>,
) -> Result<R, String> {
    let preferred = state.preferred.lock().map_err(|e| e.to_string())?.clone();
    let mut guard = state.mouse.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        let mouse = match preferred {
            Some(id) => {
                // O mouse escolhido sumiu: cai na tela "não encontrado" e o `connect`
                // seguinte escolhe outro e relê os perfis. Nunca troca em silêncio aqui.
                let connected: Vec<&str> = salamandra_hid::connected_models()
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .map(|m| m.id)
                    .collect();
                if !is_plugged(&id, &connected) {
                    return Err(Error::NotFound.to_string());
                }
                salamandra_hid::open_model(Some(&id))
            }
            None => salamandra_hid::open_model(None),
        };
        *guard = Some(mouse.map_err(|e| e.to_string())?);
    }
    let result = f(guard.as_mut().expect("aberto acima"));
    if let Err(Error::Io(_)) = &result {
        *guard = None;
    }
    result.map_err(|e| e.to_string())
}

fn read_all(mouse: &mut Mouse<HidTransport>) -> Result<Snapshot, Error> {
    let release = mouse.transport_mut().release();
    Ok(Snapshot {
        model: ModelInfo::of(mouse.spec(), release),
        active_profile: mouse.active_profile()?,
        profiles: (0..map::PROFILE_COUNT)
            .map(|p| mouse.profile(p))
            .collect::<Result<_, _>>()?,
        // Posição que o modelo não tem fica vazia, sem ler.
        macros: (0..map::MACRO_COUNT)
            .map(|i| {
                if i < mouse.spec().macro_slots {
                    mouse.macro_events(i)
                } else {
                    Ok(Vec::new())
                }
            })
            .collect::<Result<_, _>>()?,
    })
}

fn remember(state: &AppState, snapshot: &Snapshot) {
    if let Ok(mut loaded) = state.loaded.lock() {
        *loaded = snapshot.profiles.clone();
    }
}

/// Mouse conectado que o app sabe configurar, para o seletor da tela.
#[derive(Serialize)]
struct ConnectedMouse {
    id: &'static str,
    name: &'static str,
}

#[tauri::command]
fn connected_mice() -> Result<Vec<ConnectedMouse>, String> {
    Ok(salamandra_hid::connected_models()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|m| ConnectedMouse {
            id: m.id,
            name: m.name,
        })
        .collect())
}

/// Lê o mouse. Com `model`, passa a usar esse modelo (escolha do seletor).
/// Com mais de um conectado e nenhuma escolha, fica com o primeiro.
/// O estado só muda depois da leitura dar certo: em falha, segue o mouse anterior.
#[tauri::command]
fn connect(state: State<AppState>, model: Option<String>) -> Result<Snapshot, String> {
    let wanted = match model {
        Some(id) => {
            let connected: Vec<&str> = salamandra_hid::connected_models()
                .map_err(|e| e.to_string())?
                .into_iter()
                .map(|m| m.id)
                .collect();
            if !is_plugged(&id, &connected) {
                // Não é "mouse não encontrado": o mouse atual segue em uso.
                return Err(
                    Error::Invalid(format!("o mouse {id} não está mais conectado")).to_string(),
                );
            }
            Some(id)
        }
        None => state.preferred.lock().map_err(|e| e.to_string())?.clone(),
    };
    let choice = salamandra_hid::pick_model(wanted.as_deref());
    // `pick_model` já levou em conta SALAMANDRA_MODEL: `None` aqui é "o único mouse".
    let mut mouse = salamandra_hid::open_model(choice.as_deref()).map_err(|e| e.to_string())?;
    let snapshot = read_all(&mut mouse).map_err(|e| e.to_string())?;
    // Guarda o modelo aberto de fato, para reabrir sempre o mesmo e nunca outro.
    let opened = mouse.spec().id.to_string();
    *state.mouse.lock().map_err(|e| e.to_string())? = Some(mouse);
    *state.preferred.lock().map_err(|e| e.to_string())? = Some(opened);
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
fn factory_defaults(state: State<AppState>, index: usize) -> Result<Profile, String> {
    if index >= map::PROFILE_COUNT {
        return Err("perfil inválido".into());
    }
    with_mouse(&state, |m| Ok(factory_profile_for(m.spec(), index)))
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

fn custom_actions_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("custom-actions.json"))
}

/// Ações customizadas dos atalhos do mouse (arquivo vazio ou ausente: nenhuma).
#[tauri::command]
fn custom_actions(app: tauri::AppHandle) -> Result<salamandra_core::custom::CustomActions, String> {
    let path = custom_actions_path(&app)?;
    salamandra_service::load_custom_actions(&path)
        .map(|a| a.unwrap_or_else(salamandra_core::custom::CustomActions::new))
        .map_err(|e| e.to_string())
}

/// Valida, grava no arquivo e passa a valer no serviço na hora.
#[tauri::command]
fn set_custom_actions(
    app: tauri::AppHandle,
    actions: salamandra_core::custom::CustomActions,
) -> Result<(), String> {
    let path = custom_actions_path(&app)?;
    salamandra_service::set_custom_actions(Some(actions.clone())).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&actions).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
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

fn read_profile_file(path: &str, spec: &salamandra_core::DeviceSpec) -> Result<Profile, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let file: ProfileFile =
        serde_json::from_str(&text).map_err(|_| "arquivo de perfil inválido")?;
    if file.format != PROFILE_FORMAT || file.version != 1 {
        return Err("arquivo de perfil de outro formato ou versão".into());
    }
    if file.profile.buttons.len() != map::BUTTON_COUNT {
        return Err("arquivo de perfil com número errado de botões".into());
    }
    file.profile.validate_for(spec).map_err(|e| e.to_string())?;
    Ok(file.profile)
}

#[tauri::command]
fn import_profile(state: State<AppState>, path: String) -> Result<Profile, String> {
    let spec = with_mouse(&state, |m| Ok(m.spec()))?;
    read_profile_file(&path, spec)
}

/// "Reset all" do oficial: os 5 perfis voltam para o padrão de fábrica.
#[tauri::command]
fn reset_all(state: State<AppState>) -> Result<Snapshot, String> {
    let snapshot = with_mouse(&state, |m| {
        for p in 0..map::PROFILE_COUNT {
            let old = m.profile(p)?;
            let spec = m.spec();
            m.update_profile(p, &old, &factory_profile_for(spec, p))?;
        }
        read_all(m)
    })?;
    remember(&state, &snapshot);
    Ok(snapshot)
}

fn show_main(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// Ícone na bandeja com "Abrir" e "Sair", como o software oficial.
fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    // Textos provisórios: o front manda os traduzidos (`set_tray_text`) ao carregar.
    let open = MenuItem::with_id(app, "open", "Abrir Salamandra", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;
    *app.state::<AppState>()
        .tray_items
        .lock()
        .map_err(|e| tauri::Error::Io(std::io::Error::other(e.to_string())))? =
        Some((open.clone(), quit.clone()));
    let mut tray = TrayIconBuilder::new()
        .tooltip("Salamandra")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main(app),
            "quit" => {
                STOP_SERVICE.store(true, std::sync::atomic::Ordering::Relaxed);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        });
    // No macOS a bandeja usa silhueta preta (template), que o sistema pinta
    // para o tema claro ou escuro. Nos outros sistemas vai o ícone colorido.
    #[cfg(target_os = "macos")]
    {
        let template =
            tauri::image::Image::from_bytes(include_bytes!("../icons/tray-template.png"))?;
        tray = tray.icon(template).icon_as_template(true);
    }
    #[cfg(not(target_os = "macos"))]
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// O front manda os textos já traduzidos do menu da bandeja.
#[tauri::command]
fn set_tray_text(state: State<AppState>, open: String, quit: String) -> Result<(), String> {
    let guard = state.tray_items.lock().map_err(|e| e.to_string())?;
    let (o, q) = guard.as_ref().ok_or("bandeja ainda não criada")?;
    o.set_text(open).map_err(|e| e.to_string())?;
    q.set_text(quit).map_err(|e| e.to_string())
}

static STOP_SERVICE: AtomicBool = AtomicBool::new(false);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            setup_tray(app)?;
            // Ações customizadas salvas valem desde a abertura; arquivo ruim só é avisado.
            let loaded = custom_actions_path(app.handle()).and_then(|p| {
                salamandra_service::load_custom_actions(&p).map_err(|e| e.to_string())
            });
            match loaded {
                Ok(actions) => {
                    let _ = salamandra_service::set_custom_actions(actions);
                }
                Err(e) => {
                    let _ = app.handle().emit("service-error", e);
                }
            }
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let errors = handle.clone();
                salamandra_service::run(
                    &STOP_SERVICE,
                    || {
                        let state = handle.state::<AppState>();
                        let choice = state.preferred.lock().ok().and_then(|p| p.clone());
                        choice
                    },
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
        .on_window_event(|window, event| {
            // Como o oficial: fechar a janela só esconde; o serviço de atalhos segue.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            connect,
            connected_mice,
            save_profile,
            set_active_profile,
            set_macro,
            factory_defaults,
            key_names,
            macro_names,
            set_macro_names,
            custom_actions,
            set_custom_actions,
            export_profile,
            import_profile,
            reset_all,
            backup,
            restore,
            set_tray_text
        ])
        .build(tauri::generate_context!())
        .expect("falha ao iniciar o app")
        .run(|app, event| {
            // macOS: clicar no ícone do Dock reabre a janela escondida.
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = event {
                show_main(app);
            }
            #[cfg(not(target_os = "macos"))]
            let _ = (app, event);
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reconnect_only_reopens_the_chosen_mouse_while_it_is_plugged() {
        assert!(is_plugged("m901-1", &["m711-cobra", "m901-1"]));
        assert!(!is_plugged("m901-1", &["m711-cobra"]));
        assert!(!is_plugged("m901-1", &[]));
    }

    #[test]
    fn profile_file_round_trip() {
        let path = std::env::temp_dir().join("salamandra-perfil-teste.json");
        let path = path.to_string_lossy().to_string();
        let profile = salamandra_core::defaults::factory_profile(2);
        export_profile(path.clone(), profile.clone()).unwrap();
        let spec = &salamandra_core::spec::M901_1;
        assert_eq!(read_profile_file(&path, spec).unwrap(), profile);

        let mut broken = profile.clone();
        broken.buttons[0] = salamandra_core::ButtonAction::Disabled;
        export_profile(path.clone(), broken).unwrap();
        assert!(
            read_profile_file(&path, spec).is_err(),
            "sem clique esquerdo não entra"
        );

        std::fs::write(&path, "{\"format\":\"outro\"}").unwrap();
        assert!(read_profile_file(&path, spec).is_err());
        let _ = std::fs::remove_file(path);
    }
}
