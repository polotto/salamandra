//! Descrição de cada modelo suportado. O protocolo é o mesmo (enquadramento
//! `02 F3 addr len`, `F1`, `F5`); o que muda por modelo fica aqui como dado.
//! Origem de cada campo em `docs/models/` e `docs/work/multi-modelo/RE-FINDINGS.md`.

use crate::Error;

/// Se o modelo já foi comprovado no mouse físico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Status {
    /// Testado no mouse físico pelo autor.
    Tested,
    /// Mapeado a partir do software oficial e de documentação pública, nunca
    /// comprovado no mouse físico.
    Untested,
}

/// Codificação do DPI. O oficial escolhe pelo sensor (`FUN_00457b30`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DpiCodec {
    /// Sensor 3360 (M901-1): 100 a 24000 em passos de 100.
    Sensor3360,
    /// Sensor 3325 (M711 Cobra): 100 a 5000 em passos de 100 e 5200 a 10000
    /// em passos de 200.
    Sensor3325,
    /// Sensor 3327 (M908 Impact): 100 a 6200 em passos de 100 e 6400 a 12400
    /// em passos de 200.
    Sensor3327,
}

/// Codificação do modo de LED (`FUN_00456d90`, por família).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedCodec {
    /// Família 0 (M901-1, M711 Cobra): `modo velocidade sub`, com modo 1 fixo,
    /// 2 respirar e 4 piscar.
    Family0,
    /// Família 3 (M908 Impact): modo 1 com `sub` 2 fixo, 4 respirar e 0x10
    /// piscar; modo 0 apagado. Os valores de `sub` batem com o oficial
    /// (4, 8, 2, 0 e 1) e com a captura do mouse_m908.
    Family3,
}

impl LedCodec {
    /// Modos de LED que a interface oficial oferece, pelo nome de `LedMode`.
    pub fn modes(self) -> &'static [&'static str] {
        match self {
            Self::Family0 => &["Steady", "Breathing", "Flash", "Off"],
            // FUN_00456d90, família 3: oito posições (a ordem aqui é a de exibição).
            Self::Family3 => &[
                "Steady",
                "Breathing",
                "Rainbow",
                "Wave",
                "Alternating",
                "Reactive",
                "Flash",
                "Off",
            ],
        }
    }
}

/// Códigos do sensor 3325 para o índice 0..=50 (`DAT_004ba29f` do oficial).
/// Acima de 50 o oficial lê a mesma tabela em `índice - 25`, com a flag ligada.
const TABLE_3325: [u8; 51] = [
    0, 2, 4, 6, 8, 11, 13, 15, 18, 20, 22, 25, 27, 29, 32, 34, 36, 39, 41, 43, 46, 48, 50, 52, 55,
    57, 59, 62, 64, 66, 69, 71, 73, 76, 78, 80, 83, 85, 87, 90, 92, 94, 97, 99, 101, 104, 106, 108,
    111, 113, 115,
];

/// Códigos do sensor 3327 para o índice 0..=92 (`DAT_004ba19c` e `DAT_004ba120`
/// do oficial). Índice 0..=61: flag 0; 62..=92: flag 1. Do índice 1 em diante
/// bate com a captura do mouse_m908.
const TABLE_3327: [u8; 93] = [
    0, 4, 6, 9, 11, 13, 15, 18, 20, 22, 24, 27, 29, 31, 33, 36, 38, 40, 43, 45, 47, 49, 52, 54, 56,
    58, 61, 63, 65, 67, 70, 72, 74, 77, 79, 81, 83, 86, 88, 90, 92, 95, 97, 99, 102, 104, 106, 108,
    111, 113, 115, 117, 120, 122, 124, 127, 129, 131, 133, 135, 138, 140, 72, 74, 77, 79, 81, 83,
    86, 88, 90, 92, 95, 97, 99, 102, 104, 106, 108, 111, 113, 115, 117, 120, 122, 124, 127, 129,
    131, 133, 135, 138, 140,
];

impl DpiCodec {
    pub const fn min(self) -> u32 {
        100
    }

    pub const fn max(self) -> u32 {
        match self {
            Self::Sensor3360 => 24000,
            Self::Sensor3325 => 10000,
            Self::Sensor3327 => 12400,
        }
    }

    /// Passo válido a partir de `dpi`.
    pub const fn step_at(self, dpi: u32) -> u32 {
        match self {
            Self::Sensor3360 => 100,
            Self::Sensor3325 => {
                if dpi < 5000 {
                    100
                } else {
                    200
                }
            }
            Self::Sensor3327 => {
                if dpi < 6200 {
                    100
                } else {
                    200
                }
            }
        }
    }

    pub fn is_valid(self, dpi: u32) -> bool {
        if !(self.min()..=self.max()).contains(&dpi) {
            return false;
        }
        match self {
            Self::Sensor3360 => dpi.is_multiple_of(100),
            Self::Sensor3325 => {
                if dpi <= 5000 {
                    dpi.is_multiple_of(100)
                } else {
                    (dpi - 5000).is_multiple_of(200)
                }
            }
            Self::Sensor3327 => {
                if dpi <= 6200 {
                    dpi.is_multiple_of(100)
                } else {
                    (dpi - 6200).is_multiple_of(200)
                }
            }
        }
    }

    pub fn describe(self) -> &'static str {
        match self {
            Self::Sensor3360 => "100 a 24000 em passos de 100",
            Self::Sensor3325 => "100 a 5000 em passos de 100 e 5200 a 10000 em passos de 200",
            Self::Sensor3327 => "100 a 6200 em passos de 100 e 6400 a 12400 em passos de 200",
        }
    }

    /// Todos os valores de DPI que o sensor aceita, em ordem.
    pub fn values(self) -> Vec<u32> {
        let mut out = Vec::new();
        let mut dpi = self.min();
        while dpi <= self.max() {
            out.push(dpi);
            dpi += self.step_at(dpi);
        }
        out
    }

    /// `(byte, flag)` de um eixo.
    pub fn encode(self, dpi: u32) -> Result<(u8, u8), Error> {
        if !self.is_valid(dpi) {
            return Err(Error::Invalid(format!(
                "DPI {dpi} fora do permitido: {}",
                self.describe()
            )));
        }
        Ok(match self {
            Self::Sensor3360 => {
                // `índice = DPI / 100`. Até 0x79: `índice - 1`, flag 0. Acima: `índice - 0x3D`, flag 1.
                let index = dpi / 100;
                if index <= 0x79 {
                    ((index - 1) as u8, 0)
                } else {
                    ((index - 0x3D) as u8, 1)
                }
            }
            Self::Sensor3325 => {
                let index = if dpi <= 5000 {
                    dpi / 100
                } else {
                    50 + (dpi - 5000) / 200
                } as usize;
                if index < TABLE_3325.len() {
                    (TABLE_3325[index], 0)
                } else {
                    (TABLE_3325[index - 25], 1)
                }
            }
            Self::Sensor3327 => {
                let index = if dpi <= 6200 {
                    dpi / 100 - 1
                } else {
                    62 + (dpi - 6400) / 200
                } as usize;
                (TABLE_3327[index], (index >= 62) as u8)
            }
        })
    }

    /// Inverso de [`encode`](Self::encode). Código desconhecido vira 0.
    pub fn decode(self, byte: u8, flag: u8) -> u32 {
        if self == Self::Sensor3360 {
            return (byte as u32 + if flag == 0 { 1 } else { 0x3D }) * 100;
        }
        self.values()
            .into_iter()
            .find(|&dpi| self.encode(dpi) == Ok((byte, flag.min(1))))
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub vendor_id: u16,
    pub product_id: u16,
    /// Família do oficial (`FUN_00431a30`).
    pub family: u8,
    /// Modelo do sensor (`FUN_004314d0`).
    pub sensor: u16,
    pub status: Status,
    /// De onde vieram os dados deste modelo.
    pub source: &'static str,
    /// Índices de botão que existem no mouse, no espaço de 0 a 19 do M901-1.
    pub buttons: &'static [usize],
    pub dpi: DpiCodec,
    pub led: LedCodec,
    /// Posições de macro que o modelo tem (de 1 a [`crate::map::MACRO_COUNT`]).
    pub macro_slots: usize,
}

impl DeviceSpec {
    pub fn is_tested(&self) -> bool {
        self.status == Status::Tested
    }

    /// Aviso para mostrar antes de gravar num modelo nunca testado no físico.
    pub fn warning(&self) -> Option<String> {
        (!self.is_tested()).then(|| {
            format!(
                "AVISO: {} nunca foi testado no mouse físico. Os valores vêm do software oficial e de documentação pública. Faça um backup antes (salamandra backup) e relate o resultado.",
                self.name
            )
        })
    }
}

pub const M901_1: DeviceSpec = DeviceSpec {
    id: "m901-1",
    name: "Redragon M901-1 Perdition 2",
    vendor_id: 0x04D9,
    product_id: 0xFC40,
    family: 0,
    sensor: 3360,
    status: Status::Tested,
    source: "software oficial (engenharia reversa) e teste no mouse físico",
    buttons: &[
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    ],
    dpi: DpiCodec::Sensor3360,
    led: LedCodec::Family0,
    macro_slots: 15,
};

pub const M711_COBRA: DeviceSpec = DeviceSpec {
    id: "m711-cobra",
    name: "Redragon M711 Cobra",
    vendor_id: 0x04D9,
    product_id: 0xFC30,
    family: 0,
    sensor: 3325,
    status: Status::Untested,
    source: "software oficial (RDCfg.exe, família 0, sensor 3325) e captura do mouse_m908",
    buttons: &[0, 1, 2, 3, 4, 5, 6, 7, 10, 11],
    dpi: DpiCodec::Sensor3325,
    led: LedCodec::Family0,
    macro_slots: 15,
};

pub const M719_INVADER: DeviceSpec = DeviceSpec {
    id: "m719-invader",
    name: "Redragon M719 Invader",
    vendor_id: 0x04D9,
    product_id: 0xFC4F,
    family: 2,
    sensor: 3325,
    status: Status::Untested,
    source: "software oficial (RDCfg.exe, família 2, sensor 3325) e captura do mouse_m908",
    buttons: &[0, 1, 2, 3, 4, 5, 6, 7, 10, 11],
    dpi: DpiCodec::Sensor3325,
    led: LedCodec::Family0,
    // O mouse_m908 só conseguiu gravar uma macro neste modelo.
    macro_slots: 1,
};

pub const M908_IMPACT: DeviceSpec = DeviceSpec {
    id: "m908-impact",
    name: "Redragon M908 Impact",
    vendor_id: 0x04D9,
    product_id: 0xFC4D,
    family: 3,
    sensor: 3327,
    status: Status::Untested,
    source: "software oficial (RDCfg.exe, família 3, sensor 3327) e captura do mouse_m908",
    buttons: &[
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    ],
    dpi: DpiCodec::Sensor3327,
    led: LedCodec::Family3,
    macro_slots: 15,
};

/// Modelos conhecidos. O primeiro é o padrão.
pub const MODELS: &[DeviceSpec] = &[M901_1, M711_COBRA, M719_INVADER, M908_IMPACT];

pub fn by_product_id(vendor_id: u16, product_id: u16) -> Option<&'static DeviceSpec> {
    MODELS
        .iter()
        .find(|m| m.vendor_id == vendor_id && m.product_id == product_id)
}

pub fn by_id(id: &str) -> Option<&'static DeviceSpec> {
    MODELS.iter().find(|m| m.id.eq_ignore_ascii_case(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pares (DPI, byte, flag) do sensor 3325 conferidos em duas fontes: a
    /// tabela do oficial e a captura do mouse_m908.
    #[test]
    fn sensor_3325_known_codes() {
        let c = DpiCodec::Sensor3325;
        assert_eq!(c.encode(100).unwrap(), (0x02, 0));
        assert_eq!(c.encode(800).unwrap(), (0x12, 0));
        assert_eq!(c.encode(1000).unwrap(), (0x16, 0));
        assert_eq!(c.encode(5000).unwrap(), (0x73, 0));
        assert_eq!(c.encode(5200).unwrap(), (59, 1));
        assert_eq!(c.encode(10000).unwrap(), (115, 1));
    }

    #[test]
    fn sensor_3325_round_trip_and_limits() {
        assert_eq!(DpiCodec::Sensor3325.values().len(), 75);
        assert_eq!(DpiCodec::Sensor3360.values().len(), 240);
        let c = DpiCodec::Sensor3325;
        let mut valid = 0;
        for dpi in (100..=10000).step_by(100) {
            if c.is_valid(dpi) {
                valid += 1;
                let (b, f) = c.encode(dpi).unwrap();
                assert_eq!(c.decode(b, f), dpi);
            } else {
                assert!(c.encode(dpi).is_err());
            }
        }
        assert_eq!(valid, 75);
        assert!(c.encode(5100).is_err());
        assert!(c.encode(10200).is_err());
        assert!(c.encode(50).is_err());
    }

    #[test]
    fn sensor_3360_keeps_old_rule() {
        let c = DpiCodec::Sensor3360;
        assert_eq!(c.encode(1000).unwrap(), (0x09, 0));
        assert_eq!(c.encode(12200).unwrap().1, 1);
        assert!(c.encode(24100).is_err());
        assert_eq!(c.decode(0x50, 1), 14100);
    }

    #[test]
    fn sensor_3327_known_codes() {
        let c = DpiCodec::Sensor3327;
        assert_eq!(c.encode(100).unwrap(), (0, 0));
        assert_eq!(c.encode(200).unwrap(), (4, 0));
        assert_eq!(c.encode(6200).unwrap(), (140, 0));
        assert_eq!(c.encode(6400).unwrap(), (72, 1));
        assert_eq!(c.encode(12400).unwrap(), (140, 1));
        assert_eq!(c.values().len(), 93);
        for dpi in c.values() {
            let (b, f) = c.encode(dpi).unwrap();
            assert_eq!(c.decode(b, f), dpi);
        }
        assert!(c.encode(6300).is_err());
        assert!(c.encode(12600).is_err());
    }

    #[test]
    fn models_are_unique() {
        for (i, a) in MODELS.iter().enumerate() {
            for b in &MODELS[i + 1..] {
                assert_ne!(a.id, b.id);
                assert_ne!((a.vendor_id, a.product_id), (b.vendor_id, b.product_id));
            }
        }
        assert_eq!(by_product_id(0x04D9, 0xFC30).unwrap().id, "m711-cobra");
        assert!(M711_COBRA.buttons.contains(&0));
    }
}
