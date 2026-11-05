export type Lang = "tr" | "en";

const dict = {
  tr: {
    slogan: "Windows için Dinamik Ada ve dock",
    startWithWindows: "Windows ile başlat",
    install: "Kurun",
    licenseNote: "Kurun düğmesine basarak kullanım koşullarını kabul edersiniz.",
    preparing: "Hazırlanıyor…",
    installing: "Kuruluyor…",
    updating: "Güncelleniyor…",
    update: "Güncelleyin",
    updated: "MacDows güncellendi.",
    finishing: "Tamamlanıyor…",
    doneTitle: "Hazır",
    doneText: "MacDows kuruldu.",
    open: "MacDows'u açın",
    errorTitle: "Kurulum tamamlanamadı",
    close: "Kapatın",
    retry: "Yeniden deneyin",
    alreadyInstalled: "MacDows zaten kurulu.",
    reinstall: "Yeniden kurun",
    newerInstalled: "Daha yeni bir sürüm kurulu",
    upgradeText: "Kurulu sürüm",
  },
  en: {
    slogan: "Dynamic Island and dock for Windows",
    startWithWindows: "Start with Windows",
    install: "Install",
    licenseNote: "By clicking Install you accept the license terms.",
    preparing: "Preparing…",
    installing: "Installing…",
    updating: "Updating…",
    update: "Update",
    updated: "MacDows is updated.",
    finishing: "Finishing…",
    doneTitle: "Ready",
    doneText: "MacDows is installed.",
    open: "Open MacDows",
    errorTitle: "Setup could not finish",
    close: "Close",
    retry: "Try again",
    alreadyInstalled: "MacDows is already installed.",
    reinstall: "Reinstall",
    newerInstalled: "A newer version is installed",
    upgradeText: "Installed version",
  },
} as const;

export type Key = keyof (typeof dict)["en"];

export function text(lang: Lang, key: Key): string {
  return dict[lang][key];
}
