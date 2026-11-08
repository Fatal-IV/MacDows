import { useEffect, useState, type ReactNode } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { setSetting, type Settings as AppSettings } from "../bridge";
import { initTheme } from "../appearance/theme";
import { useSettings, useT } from "../i18n";
import { checkForUpdate, installUpdate, useAppVersion, useUpdateStatus, type UpdateStatus } from "../updates";
import { Button, Group, Row, Select, Toggle } from "../ui/controls";
import WindowControls from "../ui/WindowControls";
import { FoldCard } from "./FoldCard";
import Appearance from "./Appearance";
import IslandSettings from "./IslandSettings";
import { VisibilityPicker } from "./VisibilityPicker";
import WhatsNewDialog from "./WhatsNewDialog";
import "./Settings.css";

const LANGUAGES = ["en", "tr"];

function statusLine(t: (key: string) => string, status: UpdateStatus): string {
  switch (status.kind) {
    case "idle":
      return t("update.idle");
    case "checking":
      return t("update.checking");
    case "up_to_date":
      return t("update.upToDate");
    case "available":
      return t("update.available").replace("{v}", status.version);
    case "installing":
      return status.percent == null ? t("update.installing") : `${t("update.installing")} ${status.percent}%`;
    case "error":
      return t(`update.error.${status.message}`);
  }
}

function DockSettings({ s, t }: { s: AppSettings; t: (key: string) => string }) {
  return (
    <>
      <FoldCard title={t("dock.section.taskbar")}>
        <Row title={t("settings.replaceTaskbar")} description={t("settings.replaceTaskbarHelp")}>
          <Toggle label={t("settings.replaceTaskbar")} checked={s.replace_taskbar} onChange={(v) => setSetting("replace_taskbar", v)} />
        </Row>
      </FoldCard>
        <FoldCard title={t("dock.section.visibility")}>
          <VisibilityPicker value={s.dock_visibility} onChange={(v) => setSetting("dock_visibility", v)} t={t} edge="bottom" />
        </FoldCard>
        <FoldCard title={t("dock.section.apps")}>
          <Row title={t("settings.showPinned")} description={t("settings.showPinnedHelp")}>
            <Toggle label={t("settings.showPinned")} checked={s.show_pinned_apps} onChange={(v) => setSetting("show_pinned_apps", v)} />
          </Row>
          {s.hidden_pinned.length > 0 && (
            <Row title={t("settings.hiddenPinned")} description={t("settings.hiddenPinnedHelp").replace("{n}", String(s.hidden_pinned.length))}>
              <Button onClick={() => setSetting("hidden_pinned", [])}>{t("settings.hiddenPinnedReset")}</Button>
            </Row>
          )}
          <Row title={t("settings.showDrives")} description={t("settings.showDrivesHelp")}>
            <Toggle label={t("settings.showDrives")} checked={s.dock.show_drives} onChange={(v) => setSetting("dock", { ...s.dock, show_drives: v })} />
          </Row>
          <Row title={t("settings.showDesktop")} description={t("settings.showDesktopHelp")}>
            <Toggle label={t("settings.showDesktop")} checked={s.show_desktop_button} onChange={(v) => setSetting("show_desktop_button", v)} />
          </Row>
        </FoldCard>
        <FoldCard title={t("dock.section.behavior")}>
          <Row title={t("settings.winNumber")} description={t("settings.winNumberHelp")}>
            <Toggle label={t("settings.winNumber")} checked={s.dock.win_number} onChange={(v) => setSetting("dock", { ...s.dock, win_number: v })} />
          </Row>
          <Row title={t("settings.previews")} description={t("settings.previewsHelp")}>
            <Toggle label={t("settings.previews")} checked={s.dock.previews} onChange={(v) => setSetting("dock", { ...s.dock, previews: v })} />
          </Row>
        </FoldCard>
    </>
  );
}

/** Appearance: a switch at the top picks the Dynamic Island or the Dock; the theme they share sits under it. */
function AppearancePage({ s, t }: { s: AppSettings; t: (key: string) => string }) {
  const [target, setTarget] = useState<"island" | "dock">("island");
  return (
    <>
      <div className="seg-tabs" role="tablist" aria-label={t("settings.cat.appearance")}>
        {(["island", "dock"] as const).map((id) => (
          <button key={id} type="button" role="tab" aria-selected={target === id} className={`seg-tab${target === id ? " on" : ""}`} onClick={() => setTarget(id)}>
            {t(`settings.cat.${id}`)}
          </button>
        ))}
      </div>
      <h2 className="section-title">{t("appearance.scope.shared")}</h2>
      <Appearance />
      <h2 className="section-title">{t(`settings.cat.${target}`)}</h2>
      {target === "island" ? <IslandSettings s={s} t={t} /> : <DockSettings s={s} t={t} />}
    </>
  );
}

// ---------------------------------------------------------------------------------------------
// Categories and settings
//
// Every setting says which category it belongs to; the left menu and the pages are built from that list. A new
// setting only needs an entry in ITEMS, and a category with no settings does not show up in the menu.
// ---------------------------------------------------------------------------------------------
type Category = "general" | "appearance" | "updates" | "about";

const stroke = (d: string) => <path d={d} fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" />;

const CATEGORIES: { id: Category; icon: ReactNode }[] = [
  { id: "general", icon: stroke("M5 7h8m4 0h2M5 17h2m4 0h8M13 4v6m-4 4v6") },
  {
    id: "appearance",
    // A brush: the look shared by the island and the dock.
    icon: stroke("M4 20c3 0 5-1 5-3.5a2.5 2.5 0 0 0-5 0V20zM9 15.5 19 5a1.8 1.8 0 0 1 2.5 2.5L11 17.5"),
  },
  { id: "updates", icon: stroke("M20 7v5h-5M4 17v-5h5M6.3 9A7 7 0 0 1 18 7.5M17.7 15A7 7 0 0 1 6 16.5") },
  {
    id: "about",
    icon: (
      <>
        <circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" strokeWidth="1.7" />
        {stroke("M12 11v5M12 8h.01")}
      </>
    ),
  },
];

interface Ctx {
  s: AppSettings;
  t: (key: string) => string;
  update: UpdateStatus;
  version: string;
}

interface Item {
  id: string;
  category: Category;
  render: (c: Ctx) => ReactNode;
}

const ITEMS: Item[] = [
  {
    id: "general",
    category: "general",
    render: ({ s, t }) => (
      <Group>
        <Row title={t("settings.language")}>
          <Select label={t("settings.language")} value={s.language} onChange={(v) => setSetting("language", v)} options={LANGUAGES.map((l) => ({ value: l, label: t(`lang.${l}`) }))} />
        </Row>
        <Row title={t("settings.startWithWindows")}>
          <Toggle label={t("settings.startWithWindows")} checked={s.start_with_windows} onChange={(v) => setSetting("start_with_windows", v)} />
        </Row>
      </Group>
    ),
  },
  { id: "appearance", category: "appearance", render: ({ s, t }) => <AppearancePage s={s} t={t} /> },
  {
    id: "updates",
    category: "updates",
    render: ({ s, t, update, version }) => {
      const busy = update.kind === "checking" || update.kind === "installing";
      return (
        <>
          <Group>
            <Row title={t("update.version")} description={statusLine(t, update)}>
              <strong className="version">{version}</strong>
            </Row>
            {update.kind === "available" && update.notes && (
              <div className="notes">
                <div className="notes-title">{t("update.notes")}</div>
                <pre>{update.notes}</pre>
              </div>
            )}
            <div className="ui-row actions">
              <Button disabled={busy} onClick={() => checkForUpdate().catch(() => {})}>
                {t("update.checkNow")}
              </Button>
              {update.kind === "available" && (
                <Button kind="primary" disabled={busy} onClick={() => installUpdate().catch(() => {})}>
                  {t("update.install")}
                </Button>
              )}
            </div>
          </Group>
          <Group>
            <Row title={t("update.auto")}>
              <Toggle label={t("update.auto")} checked={s.check_updates} onChange={(v) => setSetting("check_updates", v)} />
            </Row>
          </Group>
        </>
      );
    },
  },
  {
    id: "about",
    category: "about",
    render: ({ t, version }) => (
      <Group>
        <Row title="MacDows" description={t("settings.about.license")}>
          <strong className="version">{version}</strong>
        </Row>
      </Group>
    ),
  },
];

const FIRST: Category = "general";

export default function Settings() {
  const s = useSettings();
  const t = useT();
  const update = useUpdateStatus();
  const version = useAppVersion();
  // Settings always opens on General.
  const [category, setCategory] = useState<Category>(FIRST);

  // Follows the app theme: Windows's light or dark look unless the user picked one.
  useEffect(() => initTheme(), []);

  if (!s) return null;
  const used = CATEGORIES.filter((c) => ITEMS.some((i) => i.category === c.id));
  const active = used.some((c) => c.id === category) ? category : FIRST;
  const ctx: Ctx = { s, t, update, version };

  return (
    <div className="settings-app">
      <WindowControls />
      <WhatsNewDialog />
      <nav className="settings-sidebar" aria-label={t("settings.categories")}>
        {used.map((c) => (
          <button key={c.id} className={`side-item${c.id === active ? " active" : ""}`} aria-current={c.id === active ? "page" : undefined} onClick={() => setCategory(c.id)}>
            <svg width="18" height="18" viewBox="0 0 24 24" aria-hidden>
              {c.icon}
            </svg>
            <span>{t(`settings.cat.${c.id}`)}</span>
          </button>
        ))}
      </nav>
      <AnimatePresence mode="wait" initial={false}>
        <motion.main key={active} className="settings-page" initial={{ opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: 12 }} transition={{ duration: 0.15 }}>
          <header className="page-head">
            <h1>{t(`settings.cat.${active}`)}</h1>
          </header>
          <div className="page-body">
            {ITEMS.filter((i) => i.category === active).map((i) => (
              <div className="item" key={i.id}>{i.render(ctx)}</div>
            ))}
          </div>
        </motion.main>
      </AnimatePresence>
    </div>
  );
}
