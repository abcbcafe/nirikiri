use crate::category::Category;

/// All message types for the TEA architecture
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants are for future features
pub enum Message {
    // Navigation
    Quit,
    SwitchCategory(Category),

    // Output selection
    SelectOutput(usize),
    SelectNextOutput,
    SelectPrevOutput,

    // Position editing
    MoveOutput { dx: i32, dy: i32 },
    SetPosition { x: i32, y: i32 },

    // Snap positioning
    SnapLeft,   // Snap to left of other monitors
    SnapRight,  // Snap to right of other monitors
    SnapAbove,  // Snap above other monitors (centered)
    SnapBelow,  // Snap below other monitors (centered)
    Normalize,  // Shift all monitors so top-left is at (0,0)

    // Canvas controls
    PanCanvas { dx: i32, dy: i32 },
    ZoomIn,
    ZoomOut,
    ResetView,

    // Config actions
    Save,
    Reload,

    // Preview via IPC
    PreviewChanges,
    RevertPreview,

    // Error handling
    Error(String),
    ClearError,

    // Refresh outputs from IPC
    RefreshOutputs,

    // Keybindings navigation
    SelectNextKeybinding,
    SelectPrevKeybinding,
    SelectKeybinding(usize),

    // Keybindings search
    StartSearch,
    UpdateSearch(String),
    ClearSearch,

    // Keybindings editing
    StartEdit,
    CancelEdit,
    ConfirmEdit,
    AddKeybinding,
    DeleteKeybinding,

    // Appearance navigation
    SelectNextAppearanceSetting,
    SelectPrevAppearanceSetting,
    ToggleSection,

    // Appearance editing
    StartAppearanceEdit,
    CancelAppearanceEdit,
    ConfirmAppearanceEdit,
    ToggleAppearanceBool,
    IncrementValue,
    DecrementValue,
    CycleEnumForward,
    CycleEnumBackward,
    UpdateAppearanceValue(String),
}
