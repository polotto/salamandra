import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ConnectedMouse, DeviceEvent, MacroEvent, Profile, Snapshot } from "./types";

export const connect = (model?: string) => invoke<Snapshot>("connect", { model });
export const connectedMice = () => invoke<ConnectedMouse[]>("connected_mice");
export const saveProfile = (index: number, profile: Profile) =>
  invoke<Profile>("save_profile", { index, profile });
export const setActiveProfile = (index: number) => invoke<number>("set_active_profile", { index });
export const setMacro = (index: number, events: MacroEvent[]) =>
  invoke<MacroEvent[]>("set_macro", { index, events });
export const keyNames = () => invoke<[string, number][]>("key_names");
export const macroNames = () => invoke<string[]>("macro_names");
export const setMacroNames = (names: string[]) => invoke<void>("set_macro_names", { names });
export const exportProfile = (path: string, profile: Profile) => invoke<void>("export_profile", { path, profile });
export const importProfile = (path: string) => invoke<Profile>("import_profile", { path });
export const resetAll = () => invoke<Snapshot>("reset_all");
export const factoryDefaults = (index: number) => invoke<Profile>("factory_defaults", { index });
export const backup = (path: string) => invoke<void>("backup", { path });
export const restore = (path: string) => invoke<Snapshot>("restore", { path });

export const onDeviceEvent = (f: (e: DeviceEvent) => void): Promise<UnlistenFn> =>
  listen<DeviceEvent>("device-event", (e) => f(e.payload));
export const onServiceError = (f: (msg: string) => void): Promise<UnlistenFn> =>
  listen<string>("service-error", (e) => f(e.payload));
export const setTrayText = (open: string, quit: string) => invoke<void>("set_tray_text", { open, quit });
