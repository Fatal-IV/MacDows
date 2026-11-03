// The latest notification, shown inside the notch like a macOS banner.
import type { Notice } from "../bridge";
import { useT } from "../i18n";

export function NotificationBanner({ notice, onDismiss }: { notice: Notice; onDismiss: () => void }) {
  const t = useT();
  return (
    <div className="banner">
      <span className="banner-icon">{notice.icon_png_base64 ? <img src={`data:image/png;base64,${notice.icon_png_base64}`} alt="" draggable={false} /> : <span>{notice.app.slice(0, 1)}</span>}</span>
      <div className="banner-text">
        <span className="banner-app">{notice.app}</span>
        <span className="banner-title">{notice.title}</span>
        {notice.body && <span className="banner-body">{notice.body}</span>}
      </div>
      <button
        className="banner-close"
        aria-label={t("notify.dismiss")}
        onClick={(e) => {
          e.stopPropagation();
          onDismiss();
        }}
      >
        ✕
      </button>
    </div>
  );
}
