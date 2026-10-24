// Espelho dos modelos de `salamandra-core` (serde: enums como {type, value}).

export type PollingRate = "Hz125" | "Hz250" | "Hz500" | "Hz1000";

export interface DpiLevel {
  enabled: boolean;
  x: number;
  y: number;
}

export type LedMode =
  | { type: "Off" }
  | { type: "Steady" }
  | { type: "Breathing"; value: { speed: number } }
  | { type: "Flash"; value: { speed: number } }
  | { type: "Cycle" }
  | { type: "Rainbow" }
  | { type: "Wave"; value: { speed: number } }
  | { type: "Alternating"; value: { speed: number } }
  | { type: "Reactive"; value: { speed: number } };

export interface Led {
  rgb: [number, number, number];
  mode: LedMode;
  brightness: number;
}

export type FireTarget =
  | { type: "Left" }
  | { type: "Right" }
  | { type: "Middle" }
  | { type: "Key"; value: number };

export type Special =
  | "DpiCycle"
  | "DpiUp"
  | "DpiDown"
  | "ScrollUp"
  | "ScrollDown"
  | "ProfileSwitch"
  | "RateUp"
  | "RateDown"
  | "LedModeSwitch";

export type Shortcut =
  | "Cut" | "Copy" | "Paste" | "SelectAll" | "Find" | "New" | "Print" | "Save"
  | "SwitchWindow" | "CloseWindow" | "OpenFileManager" | "Run" | "ShowDesktop" | "Lock"
  | "BrowserHome" | "BrowserBack" | "BrowserForward" | "BrowserStop" | "BrowserRefresh"
  | "BrowserSearch" | "BrowserFavorites" | "Mail";

export type ButtonAction =
  | { type: "Disabled" }
  | { type: "LeftClick" }
  | { type: "RightClick" }
  | { type: "MiddleClick" }
  | { type: "Back" }
  | { type: "Forward" }
  | { type: "Key"; value: number }
  | { type: "Combo"; value: { modifiers: number; key: number } }
  | { type: "Media"; value: number }
  | { type: "Fire"; value: { target: FireTarget; times: number; delay_ms: number } }
  | { type: "Macro"; value: { index: number; repeat_while_held: boolean; count: number } }
  | { type: "Special"; value: Special }
  | { type: "Shortcut"; value: Shortcut }
  | { type: "Raw"; value: [number, number, number, number] };

export interface Profile {
  polling_rate: PollingRate | null;
  scroll_speed: number;
  active_dpi_level: number;
  dpi_levels: DpiLevel[];
  led: Led;
  buttons: ButtonAction[];
}

export type MouseButton = "Left" | "Right" | "Middle";

export type MacroEvent =
  | { type: "KeyDown"; value: number }
  | { type: "KeyUp"; value: number }
  | { type: "MouseDown"; value: MouseButton }
  | { type: "MouseUp"; value: MouseButton }
  | { type: "Delay"; value: number };

export interface ModelInfo {
  id: string;
  name: string;
  vendor_id: number;
  product_id: number;
  tested: boolean;
  source: string;
  /** Índices de botão (0 a 19) que o mouse tem. */
  buttons: number[];
  /** Todos os DPI que o sensor aceita, em ordem. */
  dpi_values: number[];
  /** Quantas macros o mouse tem. */
  macro_slots: number;
  led_modes: LedMode["type"][];
}

export interface Snapshot {
  model: ModelInfo;
  active_profile: number;
  profiles: Profile[];
  macros: MacroEvent[][];
}

export type DeviceEvent =
  | { type: "ProfileChanged"; value: number }
  | { type: "DpiLevelChanged"; value: number }
  | { type: "PollingRateChanged"; value: PollingRate | null }
  | { type: "LedModeChanged"; value: [number, number, number] }
  | { type: "ShortcutPressed"; value: Shortcut }
  | { type: "ShortcutReleased" };
