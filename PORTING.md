# Porting roadmap

A direct port of the [`react-aria`](https://react-spectrum.adobe.com/react-aria/) ecosystem to [`leptos`](https://leptos.dev), one interaction and one component at a time. Each entry links to the upstream source it mirrors; the crate column names where the port lives.

Status legend: **done** (ported and tested), **next** (queued), **planned** (not started).

## Interactions

| Hook                           | Status  | Crate                                        |
| ------------------------------ | ------- | -------------------------------------------- |
| `usePress`                     | done    | `leptos_aria_interactions::use_press`        |
| `useFocus`                     | done    | `leptos_aria_interactions::use_focus`        |
| `useFocusWithin`               | done    | `leptos_aria_interactions::use_focus_within` |
| `useHover`                     | done    | `leptos_aria_interactions::use_hover`        |
| `useFocusable`                 | next    | `leptos_aria_interactions`                   |
| `useKeyboard` / `useClickable` | next    | `leptos_aria_interactions`                   |
| `useLongPress`                 | planned | `leptos_aria_interactions`                   |
| `useScroll`                    | planned | `leptos_aria_interactions`                   |
| `useDrag` / `useDrop`          | planned | `leptos_aria_interactions`                   |
| `useMove`                      | planned | `leptos_aria_interactions`                   |
| `useDismiss`                   | planned | `leptos_aria_interactions`                   |

## Utilities

| Utility                                    | Status     | Crate                                        |
| ------------------------------------------ | ---------- | -------------------------------------------- |
| `focusWithoutScrolling`                    | done       | `leptos_aria_utils::focus_without_scrolling` |
| `runAfterTransition`                       | done       | `leptos_aria_utils::run_after_transition`    |
| `isVirtualClick` / `isVirtualPointerEvent` | done       | `leptos_aria_utils`                          |
| `useGlobalListeners`                       | done       | `leptos_aria_utils::GlobalListeners`         |
| `platform` detection                       | done       | `leptos_aria_utils`                          |
| `getSyntheticLinkProps` / `openLink`       | planned    | `leptos_aria_utils`                          |
| `useLabels` / `filterDOMProps`             | planned    | `leptos_aria_utils`                          |
| `mergeRefs` equivalent                     | planned    | `leptos_aria_utils`                          |
| `useDeepMemo` equivalent                   | not needed | idiomatic leptos signals memoize             |

## Focus

| Feature                            | Status  | Crate                         |
| ---------------------------------- | ------- | ----------------------------- |
| `focusSafely`                      | next    | new `leptos_aria_focus` crate |
| `useFocusManagement` / focus scope | planned | `leptos_aria_focus`           |
| `useAutofocus`                     | planned | `leptos_aria_focus`           |
| `useChildNodes` / `useCollection`  | planned | `leptos_aria_utils`           |

## Selection

| Feature                     | Status  | Crate                             |
| --------------------------- | ------- | --------------------------------- |
| `useTypeSelect`             | planned | new `leptos_aria_selection` crate |
| `SelectionManager`          | planned | `leptos_aria_selection`           |
| `useMultipleSelectionState` | planned | `leptos_aria_selection`           |

## Components

| Component                                                   | Status  | Crate                          |
| ----------------------------------------------------------- | ------- | ------------------------------ |
| `Button` (native element)                                   | done    | `leptos_aria_button::Button`   |
| `Button` (custom element types, `useButton` anchors/inputs) | next    | `leptos_aria_button`           |
| `ToggleButton` / `useToggle`                                | next    | new `leptos_aria_toggle` crate |
| `Form` / `useFormReset`                                     | planned | `leptos_aria_form`             |
| `TextField` / `useTextField`                                | planned | `leptos_aria_text_field`       |
| `TextArea`                                                  | planned | `leptos_aria_text_field`       |
| `SearchField`                                               | planned | `leptos_aria_search_field`     |
| `NumberField`                                               | planned | `leptos_aria_number_field`     |
| `Checkbox` / `useCheckbox`                                  | planned | `leptos_aria_checkbox`         |
| `CheckboxGroup`                                             | planned | `leptos_aria_checkbox`         |
| `RadioGroup` / `useRadioGroup`                              | planned | `leptos_aria_radio`            |
| `Slider` / `useSlider`                                      | planned | `leptos_aria_slider`           |
| `Switch` (deprecated upstream in favour of `ToggleButton`)  | skipped | —                              |
| `SelectBox` / `useSelectBox`                                | planned | `leptos_aria_select_box`       |
| `ComboBox` / `useComboBox`                                  | planned | `leptos_aria_combo_box`        |
| `ListBox` / `useListBox`                                    | planned | `leptos_aria_list_box`         |
| `Menu` / `useMenu`                                          | planned | `leptos_aria_menu`             |
| `Tabs` / `useTabs`                                          | planned | `leptos_aria_tabs`             |
| `TagGroup` / `useTagGroup`                                  | planned | `leptos_aria_tag`              |
| `Toolbar`                                                   | planned | `leptos_aria_toolbar`          |
| `Tooltip` / `useTooltip`                                    | planned | `leptos_aria_tooltip`          |
| `Popover` / `usePopover`                                    | planned | `leptos_aria_overlay`          |
| `Modal` / `useModal`                                        | planned | `leptos_aria_overlay`          |
| `ModalOverlay`                                              | planned | `leptos_aria_overlay`          |
| `OverlayArrow`                                              | planned | `leptos_aria_overlay`          |
| `Dialog` / `useDialog`                                      | planned | `leptos_aria_dialog`           |
| `Calendar` / `useCalendar`                                  | planned | `leptos_aria_calendar`         |
| `DatePicker` / `useDatePicker`                              | planned | `leptos_aria_date_picker`      |
| `DateRangePicker`                                           | planned | `leptos_aria_date_picker`      |
| `TimeField` / `useTimeField`                                | planned | `leptos_aria_date_picker`      |
| `FileTrigger`                                               | planned | `leptos_aria_file_trigger`     |
| `Grid` / `useGrid`                                          | planned | `leptos_aria_grid`             |
| `Table` / `useTable`                                        | planned | `leptos_aria_table`            |
| `Tree` / `useTree`                                          | planned | `leptos_aria_tree`             |
| `Meter` / `useMeter`                                        | planned | `leptos_aria_progress`         |
| `ProgressBar` / `useProgressBar`                            | planned | `leptos_aria_progress`         |
| `Toast` / `useToast`                                        | planned | `leptos_aria_toast`            |
| `DropZone` / `useDropZone`                                  | planned | `leptos_aria_dnd`              |
| `DragAndDrop` (newer API)                                   | planned | `leptos_aria_dnd`              |
| `ColorField` / color area / wheel / sliders                 | planned | `leptos_aria_color`            |
| `Breadcrumb`                                                | planned | `leptos_aria_breadcrumbs`      |
| `Link` / `useLink`                                          | planned | `leptos_aria_link`             |
| `Heading` / `Text` primitives                               | planned | `leptos_aria_primitives`       |

## Porting conventions

- One crate per component family, matching the workspace layout, with a single `leptos_aria` facade crate re-exporting everything.
- Hooks take a `...Props` builder struct and return a `...Result` carrying the props to spread onto the target element, mirroring the upstream `useX(props) -> {xProps}` shape.
- Upstream refs become `Rc`-shared state structs whose handler methods are wrapped as cloneable callbacks.
- Upstream module-level singletons (`textSelection`, transition tracking) become thread-local state.
- Callbacks are boxed `dyn Fn` in props and `Rc`-wrapped internally; event payloads that upstream wraps (press, hover, focus) become small owned structs carrying the resolved target.
- React-specific polyfills (synthetic blur via `MutationObserver`, the React 16 `PointerEvent` fallbacks) are adapted or noted in code comments where the platform no longer needs them.
