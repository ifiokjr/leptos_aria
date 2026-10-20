---
main: patch
---

# List box options unregister when they unmount

A ListBoxOption now removes its key from the list box when it unmounts, so a list box whose options re-render, such as a filtering ComboBox, no longer piles up duplicate and stale keys. Keyboard navigation follows the options that are mounted, and aria-activedescendant clears when the focused option goes away. Each option gets a document-unique id when it registers, which stays the same while it is mounted, so ids no longer shift or collide as options come and go. ListBoxContext gains options, a map from key to ListBoxOptionEntry holding the option's id and disabled flag. A registration through register_option or register_key now lasts until the current reactive owner is cleaned up.
