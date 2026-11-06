import { useEffect, useState } from "react";
import { useT } from "../i18n";
import { takeWhatsNew, type WhatsNew } from "../updates";

/** Release notes as plain markdown: "### " headings, "- " items, anything else a paragraph. */
function Notes({ text }: { text: string }) {
  return (
    <>
      {text.split(/\r?\n/).map((line, i) => {
        const l = line.trim();
        if (!l) return null;
        if (l.startsWith("### ")) return <h4 key={i}>{l.slice(4)}</h4>;
        if (l.startsWith("- ")) return <p key={i} className="wn-item">{l.slice(2)}</p>;
        return <p key={i}>{l}</p>;
      })}
    </>
  );
}

/** Shown once, in a window of its own on top of Settings, after MacDows has been updated. */
export default function WhatsNewDialog() {
  const t = useT();
  const [news, setNews] = useState<WhatsNew | null>(null);
  useEffect(() => {
    takeWhatsNew().then(setNews).catch(() => {});
  }, []);
  useEffect(() => {
    if (!news) return;
    const key = (e: KeyboardEvent) => e.key === "Escape" && setNews(null);
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, [news]);
  if (!news) return null;
  return (
    <div className="wn-backdrop" onClick={() => setNews(null)}>
      <div className="wn-dialog" role="dialog" aria-modal="true" aria-label={t("whatsNew.title")} onClick={(e) => e.stopPropagation()}>
        <div className="wn-head">
          <div className="wn-title">{t("whatsNew.title")}</div>
          <div className="wn-version">{t("whatsNew.updated").replace("{v}", news.version)}</div>
        </div>
        <div className="wn-body">
          <Notes text={news.notes} />
        </div>
        <button className="wn-ok" autoFocus onClick={() => setNews(null)}>
          {t("whatsNew.ok")}
        </button>
      </div>
    </div>
  );
}
