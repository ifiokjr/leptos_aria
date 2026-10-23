---
main: patch
---

# Keyboard focus follows the selected tab

Arrow keys, Home and End in a tab list moved the selection but left keyboard focus on the tab you started from, so the next key press went to the wrong tab and screen readers kept announcing the old one. Focus now moves with the selection, skips disabled tabs like the selection does, and a click focuses the tab it selects (Safari does not focus a button on click). `data-focused` used to mirror the selection; it now reports the tab that has focus, so style selection with `aria-selected` instead.
