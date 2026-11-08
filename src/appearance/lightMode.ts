/** Whether the island is drawn light: the light mode, or following Windows while Windows is light. The contrast setting only matters then. */
export function isLightTheme(mode: string, systemIsDark: boolean): boolean {
  return mode === "light" || (mode === "auto" && !systemIsDark);
}
