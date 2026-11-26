# The index

Every name the facade, the components and the view export: what it is for, where the catalog
shows it and where it is declared. A dash stands where the catalog does not use the name. The
doc comment at the source says the rest.

## The facade: `morok`

| Name | What it is for | In the catalog | Source |
|---|---|---|---|
| `Application` | The running application: opens windows, holds the signal graph, runs the event loop. | `crates/catalog/src/catalog.rs` | `crates/morok/src/application.rs` |
| `Window` | A window on screen for as long as the value lives. | `crates/catalog/src/catalog.rs` | `crates/morok/src/window.rs` |
| `Styles` | The style tables an application is built with: its style sheet and the values of its tokens under each color scheme. | `crates/catalog/src/catalog.rs` | `crates/morok/src/styles.rs` |
| `Runtime` | The signal graph of an application: `morok::reactive::Runtime` whose effects act on the surfaces. | — | `crates/morok/src/lib.rs` |
| `WindowOptions` | How a window opens: its title, the size of its content, whether it blurs what is behind it. | `crates/catalog/src/catalog.rs` | `crates/platform/src/window_options.rs` |
| `Appearance` | Whether the application follows the system or shows itself light or dark. | `crates/catalog/src/catalog_window.rs` | `crates/platform/src/appearance.rs` |
| `RendererError` | Why `Application::new` could not create the renderer. | `crates/catalog/src/catalog.rs` | `crates/platform/src/renderer_error.rs` |
| `CanvasPainter` | The trait of what draws the canvases of an application. | `crates/catalog/src/disk_map_painter.rs` | `crates/platform/src/canvas/canvas_painter.rs` |
| `CanvasEncoder` | What a painter draws one canvas through: fills, the theme, the Metal encoder. | `crates/catalog/src/disk_map_painter.rs` | `crates/platform/src/canvas/canvas_encoder.rs` |
| `CanvasRegion` | Where a canvas lies on its window, and the scale of its pixels. | — | `crates/platform/src/canvas/canvas_region.rs` |
| `RingSector` | The part of a ring between two angles, which a painter fills. | `crates/catalog/src/sunburst.rs` | `crates/platform/src/canvas/ring_sector.rs` |
| `FrameImage` | The pixels of a frame, as `Window::frame_image` gives them. | `crates/catalog/tests/wire_main_path.rs` | `crates/platform/src/frame_image.rs` |
| `FrameStatistics` | What the frame loop of a window has done, as `Window::frame_statistics` gives it. | — | `crates/platform/src/frame_statistics.rs` |
| `AtlasStatistics` | The state of the glyph atlases, as `Application::atlas_statistics` gives it. | — | `crates/platform/src/atlas_statistics.rs` |
| `AtlasUsage` | How full one glyph atlas is: a part of `AtlasStatistics`. | — | `crates/platform/src/atlas_usage.rs` |
| `components` | The module of the components and the view. | `crates/catalog/src/pages` | `crates/components/src/lib.rs` |
| `core` | The module of the element tree and its vocabulary: IDs, events, keys, geometry. | `crates/catalog/src/catalog_window.rs` | `crates/core/src/lib.rs` |
| `reactive` | The module of the signal graph. | `crates/catalog/src/catalog.rs` | `crates/reactive/src/lib.rs` |
| `style` | The module of compiled styles: classes, tokens, declarations, themes. | `crates/catalog/src/token_set.rs` | `crates/style/src/lib.rs` |

## The components: `morok::components`

| Name | What it is for | In the catalog | Source |
|---|---|---|---|
| `Button` | A push button with a label. | `crates/catalog/src/pages/buttons_page.rs` | `crates/components/src/button.rs` |
| `IconButton` | A button that shows an icon; may be a toggle. | `crates/catalog/src/pages/buttons_page.rs` | `crates/components/src/icon_button.rs` |
| `TextField` | One line of text the user edits. | `crates/catalog/src/pages/text_fields_page.rs` | `crates/components/src/text_field.rs` |
| `SearchField` | One line for what to search for, with a magnifier. | `crates/catalog/src/pages/text_fields_page.rs` | `crates/components/src/search_field.rs` |
| `TextArea` | Text the user edits in lines that wrap and scroll. | `crates/catalog/src/pages/text_fields_page.rs` | `crates/components/src/text_area.rs` |
| `Checkbox` | A check box with its label. | `crates/catalog/src/pages/controls_page.rs` | `crates/components/src/checkbox.rs` |
| `Switch` | A switch with its label. | `crates/catalog/src/pages/controls_page.rs` | `crates/components/src/switch.rs` |
| `RadioGroup` | Radio buttons of which one is chosen. | `crates/catalog/src/pages/controls_page.rs` | `crates/components/src/radio_group.rs` |
| `SegmentedControl` | Segments side by side of which one is chosen. | `crates/catalog/src/pages/controls_page.rs` | `crates/components/src/segmented_control.rs` |
| `Slider` | A thumb on a rail that picks a number of a range. | `crates/catalog/src/pages/controls_page.rs` | `crates/components/src/slider.rs` |
| `ProgressBar` | A bar that shows how far a task has come. | `crates/catalog/src/pages/controls_page.rs` | `crates/components/src/progress_bar.rs` |
| `Spinner` | A ring that turns while something is under way. | `crates/catalog/src/pages/controls_page.rs` | `crates/components/src/spinner.rs` |
| `Select` | A box that holds one of several options and drops a list of them down. | `crates/catalog/src/pages/pickers_page.rs` | `crates/components/src/select.rs` |
| `Combobox` | A text field with a list of options that narrows to what is typed. | `crates/catalog/src/pages/pickers_page.rs` | `crates/components/src/combobox.rs` |
| `List` | Rows of text the user selects from, as many as there may be. | `crates/catalog/src/pages/collections_page.rs` | `crates/components/src/list.rs` |
| `RowSelection` | Whether a list or a table selects one row or several. | `crates/catalog/src/pages/collections_page.rs` | `crates/components/src/row_selection.rs` |
| `Tree` | Items in levels that open and close. | `crates/catalog/src/pages/collections_page.rs` | `crates/components/src/tree.rs` |
| `TreeItem` | An item of a tree: a key, a label and the items under it. | `crates/catalog/src/pages/collections_page.rs` | `crates/components/src/tree.rs` |
| `Tabs` | Tabs side by side of which one is selected. | `crates/catalog/src/pages/collections_page.rs` | `crates/components/src/tabs.rs` |
| `Table` | Rows of cells in columns the user resizes, moves and sorts by. | `crates/catalog/src/pages/table_page.rs` | `crates/components/src/table.rs` |
| `TableColumn` | A column of a table: its key, title, width and alignment. | `crates/catalog/src/pages/table_page.rs` | `crates/components/src/table_column.rs` |
| `CellAlignment` | The side of a column its texts stand at. | `crates/catalog/src/pages/table_page.rs` | `crates/components/src/cell_alignment.rs` |
| `TableSort` | The column the rows of a table are sorted by, and the order. | — | `crates/components/src/table_sort.rs` |
| `SortOrder` | Ascending or descending. | — | `crates/components/src/sort_order.rs` |
| `Toolbar` | The bar at the top of a window. | `crates/catalog/src/catalog_window.rs` | `crates/components/src/toolbar.rs` |
| `StatusBar` | The bar at the bottom of a window. | `crates/catalog/src/catalog_window.rs` | `crates/components/src/status_bar.rs` |
| `Sidebar` | The column at the side of a window that leads to its places. | `crates/catalog/src/catalog_window.rs` | `crates/components/src/sidebar.rs` |
| `SidebarItem` | A place a sidebar leads to: its name, icon and detail. | `crates/catalog/src/catalog_window.rs` | `crates/components/src/sidebar_item.rs` |
| `SplitView` | Panes along an axis with dividers the user drags. | `crates/catalog/src/catalog_window.rs` | `crates/components/src/split_view.rs` |
| `SplitPane` | A pane of a split view: what it shows and how long it is. | `crates/catalog/src/catalog_window.rs` | `crates/components/src/split_pane.rs` |
| `SplitAxis` | Whether a split view lays its panes side by side or one above another. | `crates/catalog/src/pages/scaffolding_page.rs` | `crates/components/src/split_axis.rs` |
| `ScrollView` | A region whose content scrolls under overlay scroll bars. | `crates/catalog/src/page.rs` | `crates/components/src/scroll_view.rs` |
| `Popover` | A floating panel that belongs to a control. | `crates/catalog/src/pages/overlays_page.rs` | `crates/components/src/popover.rs` |
| `Tooltip` | A short note that shows once the pointer has rested on a control. | `crates/catalog/src/pages/overlays_page.rs` | `crates/components/src/tooltip.rs` |
| `Menu` | A floating column of commands the kit draws. | `crates/catalog/src/pages/overlays_page.rs` | `crates/components/src/menu.rs` |
| `MenuItem` | A line of a menu: a command, a submenu or a separator. | `crates/catalog/src/pages/overlays_page.rs` | `crates/components/src/menu_item.rs` |
| `ContextMenu` | The menu a press of the secondary button opens. | `crates/catalog/src/pages/overlays_page.rs` | `crates/components/src/context_menu.rs` |
| `Dialog` | A question in the middle of the window that the user answers before going on. | `crates/catalog/src/pages/overlays_page.rs` | `crates/components/src/dialog.rs` |
| `Sheet` | A panel that hangs from the top edge of the window with a question. | `crates/catalog/src/pages/overlays_page.rs` | `crates/components/src/sheet.rs` |
| `ToastStack` | The toasts of a window, stacked in its corner. | `crates/catalog/src/pages/overlays_page.rs` | `crates/components/src/toast_stack.rs` |
| `Toast` | A short note about what just happened. | `crates/catalog/src/pages/overlays_page.rs` | `crates/components/src/toast.rs` |
| `ToastTone` | What a toast tells of: success, a warning or danger. | `crates/catalog/src/pages/overlays_page.rs` | `crates/components/src/toast_tone.rs` |
| `Image` | A picture the application added. | `crates/catalog/src/pages/images_page.rs` | `crates/components/src/image.rs` |
| `KitIcon` | The icons of the kit. | `crates/catalog/src/pages/icons_page.rs` | `crates/components/src/kit_icon.rs` |
| `KIT_TOKENS` | The tokens file of the kit, for the build script of an application. | `crates/catalog/build.rs` | `crates/components/src/stylesheets.rs` |
| `STYLESHEETS` | The stylesheets of the components, for the build script of an application. | `crates/catalog/build.rs` | `crates/components/src/stylesheets.rs` |
| `StylesheetSource` | A stylesheet of the kit: its path and its text. | `crates/catalog/src/stylesheets.rs` | `crates/components/src/stylesheet_source.rs` |

## The view: `morok::components::view`

The first four are what every component uses. The rest serve a component that opens overlays of
its own, or a host that shows the surfaces in windows of its own, as the facade does: an
application built on the facade does not need them.

| Name | What it is for | In the catalog | Source |
|---|---|---|---|
| `Mount` | What a component is given: it creates elements and reaches the signal graph. | `crates/catalog/src/page_layout.rs` | `crates/components/src/view/mount.rs` |
| `ElementBuilder` | Gives one element its properties, children and handlers. | `crates/catalog/src/page_layout.rs` | `crates/components/src/view/element_builder.rs` |
| `Property` | A value that stays, or a signal or memo a property follows. | `crates/catalog/src/page_layout.rs` | `crates/components/src/view/property.rs` |
| `ElementEvent` | What reached an element: the pointer, a key, an assistive technology, an edit, a drag. | `crates/catalog/src/pages/images_page.rs` | `crates/components/src/view/element_event.rs` |
| `Surfaces` | The surfaces of an application: the world the effects of the signal graph act on, named in the type of every handler. | `crates/catalog/src/pages/images_page.rs` | `crates/components/src/view/surfaces.rs` |
| `Action` | The boxed form of a handler that is given the signal graph. | — | `crates/components/src/view/action.rs` |
| `ScrollArea` | How far a scroll element is scrolled, as `Mount::follow_scroll` reports it. | — | `crates/components/src/view/scroll_area.rs` |
| `Overlay` | An overlay as a component asks for it: its anchor, placement and interaction. | — | `crates/components/src/view/overlay.rs` |
| `OverlayAnchor` | What an overlay is placed against: an element or a point. | — | `crates/components/src/view/overlay_anchor.rs` |
| `OverlayInteraction` | What an overlay does with the pointer and the keyboard. | — | `crates/components/src/view/overlay_interaction.rs` |
| `OverlayPlace` | Where the host placed an overlay, as signals its content follows. | — | `crates/components/src/view/overlay_place.rs` |
| `OverlayGeometry` | Where the host of a window shows an overlay. | — | `crates/components/src/view/overlay_geometry.rs` |
| `NativeMenuChoice` | The handler of a line chosen in the menu of the system. | — | `crates/components/src/view/native_menu_choice.rs` |
| `Surface` | One surface: its tree and what the components registered for its elements. | — | `crates/components/src/view/surface.rs` |
| `SurfaceId` | The handle of a surface. | — | `crates/components/src/view/surface_id.rs` |
| `SurfaceFrame` | How the frame of a surface went. | — | `crates/components/src/view/surface_frame.rs` |
| `SurfaceStage` | The stages of the frame of a surface. | — | `crates/components/src/view/surface_stage.rs` |
