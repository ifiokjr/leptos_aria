---
main: patch
---

# Reactive progress, pagination, slider, and avatar fixes

Avatar now shows its image once it loads: the image stays hidden in the tree until its load event fires, and a failed load keeps the fallback. ProgressBar and Meter recompute aria-valuenow, aria-valuetext, and the fill width whenever the value or indeterminate state changes, and ProgressBar renders the label its aria-labelledby points at. Pagination moves aria-current and data-current with the page and re-renders its buttons when the count changes. A controlled Slider value now moves the native range input. TextField renders its label only when one is given, takes id and aria_label props, and keeps a controlled value in the input after typing; Radio no longer puts its input's id on the label too. Tabs skip disabled tabs with the arrow keys, Home, End, and clicks, and keep the first enabled tab focusable while nothing is selected.
