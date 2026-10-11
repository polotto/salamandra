// Textos da interface em pt-BR e inglês. A chave é o texto em pt-BR.
const en: Record<string, string> = {
  "Perfis": "Profiles",
  "Perfil": "Profile",
  "Macros": "Macros",
  "Sobre": "About",
  "ativo": "active",
  "Alterações não gravadas": "Unsaved changes",
  "Mouse não encontrado": "Mouse not found",
  "Conecte o Redragon M901-1 Perdition 2 por USB.": "Connect the Redragon M901-1 Perdition 2 over USB.",
  "Tentar de novo": "Try again",
  "Recarregar do mouse": "Reload from mouse",
  "Perfil ativo no mouse": "Active profile on the mouse",
  "Este perfil não está ativo.": "This profile is not active.",
  "Ativar": "Activate",
  "Geral": "General",
  "DPI": "DPI",
  "Luz": "Lighting",
  "Botões": "Buttons",
  "Descartar": "Discard",
  "Gravar no mouse": "Save to mouse",
  "Padrão de fábrica": "Factory defaults",
};

class I18n {
  lang = $state<"pt" | "en">(navigator.language.toLowerCase().startsWith("pt") ? "pt" : "en");
  t = (pt: string): string => (this.lang === "en" ? (en[pt] ?? pt) : pt);
}

export const i18n = new I18n();
export const t = (pt: string) => i18n.t(pt);
