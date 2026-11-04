import { useState } from "react";

/** A localStorage-backed value. Other windows pick the change up through the `storage` event. */
export function useStored(key: string, fallback: string): [string, (v: string) => void] {
  const [value, setValue] = useState(() => localStorage.getItem(key) ?? fallback);
  return [
    value,
    (v) => {
      localStorage.setItem(key, v);
      setValue(v);
    },
  ];
}
