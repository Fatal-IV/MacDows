export type Lang = "tr" | "en";

const dict = {
  tr: {
    slogan: "Windows için notch ve dock",
    install: "Kur",
    licenseNote: "Kur'a basarak kullanım koşullarını kabul edersiniz.",
    preparing: "Hazırlanıyor…",
    installing: "Kuruluyor…",
    finishing: "Tamamlanıyor…",
    doneTitle: "Hazır",
    doneText: "MacDows kuruldu.",
    open: "MacDows'u aç",
    errorTitle: "Kurulum tamamlanamadı",
    close: "Kapat",
    retry: "Yeniden dene",
    alreadyInstalled: "MacDows zaten kurulu.",
    reinstall: "Yeniden kur",
    newerInstalled: "Daha yeni bir sürüm kurulu",
    upgradeText: "Kurulu sürüm",
  },
  en: {
    slogan: "Notch and dock for Windows",
    install: "Install",
    licenseNote: "By clicking Install you accept the license terms.",
    preparing: "Preparing…",
    installing: "Installing…",
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
