# Porting roadmap

A direct port of the [`react-aria`](https://react-spectrum.adobe.com/react-aria/) ecosystem to [`leptos`](https://leptos.dev), one interaction and one component at a time. Each entry links to the upstream source it mirrors; the crate column names where the port lives.

Status legend: **done** (ported and tested), **planned** (not started).

## Interactions

| Hook                  | Status | Crate                                        |
| --------------------- | ------ | -------------------------------------------- |
| `usePress`            | done   | `leptos_aria_interactions::use_press`        |
| `useFocus`            | done   | `leptos_aria_interactions::use_focus`        |
| `useFocusWithin`      | done   | `leptos_aria_interactions::use_focus_within` |
| `useHover`            | done   | `leptos_aria_interactions::use_hover`        |
| `useFocusable`        | done   | `leptos_aria_interactions::use_focusable`    |
| `useFocusRing`        | done   | `leptos_aria_interactions::use_focus_ring`   |
| `useKeyboard`         | done   | `leptos_aria_interactions::use_keyboard`     |
| `useLongPress`        | done   | `leptos_aria_interactions::use_long_press`   |
| `useScroll`           | done   | `leptos_aria_interactions::use_scroll`       |
| `useDrag` / `useDrop` | done   | `leptos_aria_interactions`                   |
| `useMove`             | done   | `leptos_aria_interactions::use_move`         |
| `useDismiss`          | done   | `leptos_aria_overlay::use_overlay_dismiss`   |
| `useModal`            | done   | `leptos_aria_overlay::use_modal`             |

## Utilities

| Utility                                    | Status     | Crate                                        |
| ------------------------------------------ | ---------- | -------------------------------------------- |
| `focusWithoutScrolling`                    | done       | `leptos_aria_utils::focus_without_scrolling` |
| `runAfterTransition`                       | done       | `leptos_aria_utils::run_after_transition`    |
| `isVirtualClick` / `isVirtualPointerEvent` | done       | `leptos_aria_utils`                          |
| `useGlobalListeners`                       | done       | `leptos_aria_utils::GlobalListeners`         |
| `platform` detection                       | done       | `leptos_aria_utils`                          |
| interaction modality                       | done       | `leptos_aria_utils::interaction_modality`    |
| `focusSafely`                              | done       | `leptos_aria_utils::focus_safely`            |
| `isFocusable` / `isTabbable`               | done       | `leptos_aria_utils`                          |
| `useId`                                    | done       | `leptos_aria_utils::use_id`                  |
| `useLabel` / `useLabels`                   | done       | `leptos_aria_utils`                          |
| `getSyntheticLinkProps` / `openLink`       | done       | `leptos_aria_utils::open_link`               |
| `usePreventScroll`                         | done       | `leptos_aria_overlay::use_prevent_scroll`    |
| `useFormReset`                             | done       | `leptos_aria_form::use_form_reset`           |
| `useDeepMemo` equivalent                   | not needed | idiomatic leptos signals memoize             |

## Focus

| Feature                               | Status | Crate                                |
| ------------------------------------- | ------ | ------------------------------------ |
| focus containment (scope equivalent)  | done   | `leptos_aria_overlay::use_modal`     |
| `useFocusManagement` programmatic API | done   | `leptos_aria_overlay::FocusManager`  |
| `useAutofocus`                        | done   | `leptos_aria_overlay::use_autofocus` |

## Selection

| Feature                     | Status | Crate                                                              |
| --------------------------- | ------ | ------------------------------------------------------------------ |
| `useTypeSelect`             | done   | `leptos_aria_selection` (type-ahead in `CollectionKeys` + ListBox) |
| `SelectionManager`          | done   | `leptos_aria_selection` (`SelectionState`)                         |
| `useMultipleSelectionState` | done   | `leptos_aria_selection` (`use_selection_state`)                    |

## Components

| Component                                         | Status  | Crate                                                |
| ------------------------------------------------- | ------- | ---------------------------------------------------- |
| `Button` (native element)                         | done    | `leptos_aria_button::Button`                         |
| `Button` (custom element types)                   | done    | `leptos_aria_button::AnchorButton / CustomButton`    |
| `ToggleButton` / `useToggle`                      | done    | `leptos_aria_toggle`                                 |
| `Switch` (deprecated upstream for `ToggleButton`) | skipped | —                                                    |
| `Form`                                            | done    | `leptos_aria_form::Form`                             |
| `TextField`                                       | done    | `leptos_aria_text_field::TextField`                  |
| `TextArea`                                        | done    | `leptos_aria_text_field::TextArea`                   |
| `SearchField`                                     | done    | `leptos_aria_search_field::SearchField`              |
| `NumberField`                                     | done    | `leptos_aria_number_field::NumberField`              |
| `Checkbox` / `useCheckbox`                        | done    | `leptos_aria_checkbox::Checkbox`                     |
| `CheckboxGroup`                                   | done    | `leptos_aria_checkbox::CheckboxGroup`                |
| `RadioGroup`                                      | done    | `leptos_aria_radio::RadioGroup`                      |
| `Slider`                                          | done    | `leptos_aria_slider::Slider`                         |
| `ComboBox`                                        | done    | `leptos_aria_combo_box::ComboBox`                    |
| `SelectBox`                                       | done    | `leptos_aria_select_box::SelectBox`                  |
| `ListBox`                                         | done    | `leptos_aria_list_box::ListBox`                      |
| `Menu`                                            | done    | `leptos_aria_menu::Menu` + `MenuTrigger`             |
| `Tabs`                                            | done    | `leptos_aria_tabs::Tabs`                             |
| `TagGroup`                                        | done    | `leptos_aria_tag::TagGroup`                          |
| `Toolbar`                                         | done    | `leptos_aria_toolbar::Toolbar`                       |
| `Tooltip`                                         | done    | `leptos_aria_overlay::Tooltip`                       |
| `Popover`                                         | done    | `leptos_aria_overlay::Popover` (flip/shift included) |
| `Modal` / `ModalOverlay`                          | done    | `leptos_aria_overlay::Modal`                         |
| `Dialog`                                          | done    | `leptos_aria_overlay::Dialog`                        |
| `Calendar`                                        | done    | `leptos_aria_calendar::Calendar`                     |
| `DatePicker` / `DateField`                        | done    | `leptos_aria_date_picker`                            |
| `DateRangePicker`                                 | done    | `leptos_aria_date_picker`                            |
| `TimeField`                                       | done    | `leptos_aria_time_field::TimeField`                  |
| `FileTrigger`                                     | done    | `leptos_aria_file_trigger::FileTrigger`              |
| `Grid`                                            | done    | `leptos_aria_grid::Grid`                             |
| `Table`                                           | done    | `leptos_aria_table::Table`                           |
| `Tree`                                            | done    | `leptos_aria_tree::Tree`                             |
| `Meter`                                           | done    | `leptos_aria_progress::Meter`                        |
| `ProgressBar`                                     | done    | `leptos_aria_progress::ProgressBar`                  |
| `Toast`                                           | done    | `leptos_aria_toast`                                  |
| `DropZone`                                        | done    | `leptos_aria_drop_zone::DropZone`                    |
| `DragAndDrop` (newer draggable-item API)          | done    | `leptos_aria_drop_zone::DraggableItem`               |
| `Link`                                            | done    | `leptos_aria_link::Link`                             |
| `Breadcrumb`                                      | done    | `leptos_aria_breadcrumbs`                            |
| `Heading` / `Text` / `VisuallyHidden`             | done    | `leptos_aria_primitives`                             |
| `Disclosure`                                      | done    | `leptos_aria_disclosure::Disclosure`                 |
| `Color` / `ColorField` / `ColorSwatch`            | done    | `leptos_aria_color` (incl. area/slider/wheel)        |
| `DateValue` / `DateDuration`                      | done    | `leptos_aria_date`                                   |

## Porting conventions

- One crate per component family, matching the workspace layout, with a single `leptos_aria` facade crate re-exporting everything; crate names use underscores (`leptos_aria_name`).
- Hooks take a `...Props` builder struct and return a `...Result` carrying the props to spread onto the target element, mirroring the upstream `useX(props) -> {xProps}` shape.
- Upstream refs become `Rc`/`Arc`-shared state structs whose handler methods are wrapped as cloneable callbacks.
- Upstream module-level singletons (`textSelection`, transition tracking, interaction modality) become thread-local state.
- Callbacks are boxed `dyn Fn` in props and `Rc`/`Arc`-wrapped internally; event payloads that upstream wraps (press, hover, focus) become small owned structs carrying the resolved target.
- React-specific polyfills (synthetic blur via `MutationObserver`, the React 16 `PointerEvent` fallbacks) are adapted or noted in code comments where the platform no longer needs them.
