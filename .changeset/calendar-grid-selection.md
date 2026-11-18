---
main: patch
---

# Selecting a calendar date updates the grid

Enter and Space selected the focused date in state, but the grid had already snapshotted `aria-selected` and the selected class while it was built. Those keys do not change the month, so the grid never painted the new selection and assistive technology kept announcing the previous date. Each cell now follows the focus and selection signals, so the marked date moves with the keyboard.
