# Source Processor Visualization Controls Implementation

## Overview
Comprehensive UI controls have been implemented based on the egui_graphs demo-core reference implementation. The application now features a rich, interactive interface for visualizing and manipulating the source code graph.

## New Features

### 1. **Settings System** (`src/settings.rs`)
Three modular settings structures:
- **SettingsInteraction**: Controls for user interaction (dragging, selection, hover)
- **SettingsNavigation**: Viewport controls (zoom, pan, fit-to-screen)
- **SettingsStyle**: Visual styling options (labels, themes)

### 2. **Keyboard Shortcuts** (`src/keybindings.rs`)
Comprehensive keyboard command system:
- **Tab**: Toggle sidebar visibility
- **Space**: Fit graph to screen
- **Ctrl+Space**: Toggle zoom/pan mode
- **d**: Toggle debug overlay
- **h / ?**: Show keybindings help dialog
- **Backspace**: Reset all settings to defaults
- **Escape**: Close modal dialogs

### 3. **Side Panel Controls**
Collapsible sections for fine-grained control:

#### Navigation
- **fit_to_screen**: Auto-adjust viewport to show entire graph
- **zoom_and_pan**: Manual navigation controls
- **zoom_speed**: Sensitivity slider
- **fit_to_screen_padding**: Extra space around graph

#### Layout (Force-Directed)
- **Animation Controls**:
  - `running`: Play/pause simulation
  - `dt`: Integration time step
  - `damping`: Velocity decay

- **Force Parameters**:
  - `k_scale`: Ideal edge length scale
  - `c_attract`: Attractive force multiplier
  - `c_repulse`: Repulsive force multiplier
  
- **Center Gravity**:
  - Toggle and strength control for pulling nodes toward center

#### Interaction
- **dragging_enabled**: Drag nodes to reposition
- **hover_enabled**: Highlight on mouse hover
- **node_selection**: Single-click selection
- **multi_selection**: Ctrl+click for multiple nodes

#### Style
- **dark mode**: Toggle between dark/light themes
- **labels_always**: Always show node labels

#### Selected Items
- Displays list of currently selected nodes and edges

#### Debug
- Toggle debug overlay
- Access keybindings help

### 4. **Overlay UI Elements**

#### Debug Overlay
Displays real-time information in the top-right corner:
- Node count
- Edge count
- Frames per second (FPS)

#### Toggle Buttons
Bottom-right corner buttons:
- **ℹ (Help)**: Opens keybindings modal
- **◀/▶ (Toggle)**: Show/hide sidebar

#### Keybindings Modal
Centered modal window showing all keyboard shortcuts with:
- Navigation commands
- Interface controls
- Dismissible on any key/click

### 5. **Enhanced Graph Visualization**
- Settings are applied dynamically to the graph view
- All egui_graphs features are properly configured
- Smooth integration of all controls with the force-directed layout

## Architecture

### Module Structure
```
source-processor/
├── src/
│   ├── main.rs           # Main app with UI implementation
│   ├── settings.rs       # Settings data structures
│   └── keybindings.rs    # Keyboard command dispatch
```

### Key Components

#### `SourceCodeVisualizationApp`
Main application struct with:
- Graph data (`g: Graph<(), ()>`)
- Settings structures (interaction, navigation, style)
- UI state (sidebar, overlays, modals)

#### Helper Methods
- `ui_navigation()`: Renders navigation controls
- `ui_layout()`: Renders layout controls
- `ui_interaction()`: Renders interaction controls
- `ui_style()`: Renders style controls
- `ui_selected()`: Shows selected items
- `ui_debug()`: Debug controls
- `process_keybindings()`: Keyboard event handling
- `keybindings_modal()`: Help dialog
- `debug_overlay()`: Performance metrics
- `sidebar_toggle_button()`: Toggle buttons
- `info_icon()`: Tooltip helper

## User Experience

### First Launch
- Sidebar visible by default with all controls
- Debug overlay enabled showing graph stats
- Zoom and pan enabled for manual navigation
- Labels always visible for clarity

### Navigation Flow
1. Use sidebar controls to adjust graph parameters
2. Press Tab to hide sidebar for full-screen view
3. Use keyboard shortcuts for quick actions
4. Access help with 'h' or '?' anytime

### Visual Feedback
- Hover tooltips on all controls (ℹ icons)
- Collapsible sections to reduce clutter
- Grid layout for keybindings dialog
- Consistent spacing and typography

## Technical Notes

### egui Integration
- Proper use of `egui::SidePanel` for sidebar
- `egui::Window` for modals
- `egui::Area` for floating buttons
- Responsive layout with `ScrollArea`

### State Management
- All settings have sensible defaults
- Settings are applied to graph view in each frame
- Tab key properly consumed to prevent focus issues
- Modal state prevents accidental closures

### Performance
- Debug overlay uses `debug_painter()` for minimal overhead
- Settings are built per-frame but only allocate small structures
- No unnecessary redraws or computations

## Future Enhancements

Potential additions based on demo-core features:
- Graph import/export functionality
- Multiple layout algorithms (hierarchical)
- Add/remove nodes and edges dynamically
- Status notifications system
- Metrics tracking and performance graphs
- Event logging (with events feature flag)

## Commands to Run

### Build
```bash
cd source-processor
cargo build
```

### Run
```bash
cd source-processor
cargo run
```

### Check
```bash
cd source-processor
cargo check
```

## References
- Original demo: https://github.com/blitzar-tech/egui_graphs/tree/main/crates/demo-core
- egui documentation: https://docs.rs/egui/latest/egui/
- egui_graphs: https://docs.rs/egui_graphs/latest/egui_graphs/

