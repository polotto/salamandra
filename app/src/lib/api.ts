import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DeviceEvent, MacroEvent, Profile, Snapshot } from "./types";

export const connect = () => invoke<Snapshot>("connect");
export const saveProfile = (index: number, profile: Profile) =>
  invoke<Profile>("save_profile", { index, profile });
export const setActiveProfile = (index: number) => invoke<number>("set_active_profile", { index });
export const setMacro = (index: number, events: MacroEvent[]) =>
  invoke<MacroEvent[]>("set_macro", { index, events });
export const keyNames = () => invoke<[string, number][]>("key_names");
export const factoryDefaults = (index: number) => invoke<Profile>("factory_defaults", { index });
export const backup = (path: string) => invoke<void>("backup", { path });
export const restore = (path: string) => invoke<Snapshot>("restore", { path });

export const onDeviceEvent = (f: (e: DeviceEvent) => void): Promise<UnlistenFn> =>
  listen<DeviceEvent>("device-event", (e) => f(e.payload));
export const onServiceError = (f: (msg: string) => void): Promise<UnlistenFn> =>
  listen<string>("service-error", (e) => f(e.payload));
