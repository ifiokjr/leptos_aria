---
main: minor
---

# Browser accessibility regression coverage

Production demos and documentation now run through Playwright in Chromium, Firefox, WebKit, dark mode, and a mobile viewport, with WCAG audits and keyboard journeys required by CI. Space activation no longer panics when the press ends on a document listener. Tooltips retain working event handlers, describe their focused control, dismiss on Escape, and cancel stale hover opens. Accordions link their triggers and panels; selects link their combobox and listbox; menus expose menu/menuitem semantics through the new `ListBoxRole` prop instead of nested listbox roles. Decorative separators and modal backdrops omit unsupported ARIA, destructive alerts keep sufficient text contrast, and scrollable documentation code is keyboard reachable.
