//! Module: intro
//! Purpose: Bauhaus-styled intro sequence explaining the app to users
//!
//! Displays a series of informative screens with auto-advancing timers,
//! then transitions to the main fidget experience.

use bevy::prelude::*;

use crate::resources::UiFont;

// =============================================================================
// APP STATE
// =============================================================================

/// Application state controlling whether intro or fidget mode is active.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// Intro sequence showing app information
    #[default]
    Intro,
    /// Main fidget/visualizer experience
    Fidget,
}

// =============================================================================
// INTRO STEPS
// =============================================================================

/// Individual steps in the intro sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntroStep {
    /// App title
    Title,
    /// Tagline
    Tagline,
    /// Touch instruction
    TouchInstruction,
    /// Tap instruction
    TapInstruction,
    /// Two-finger instruction
    TwoFingerInstruction,
    /// Ready prompt
    Ready,
}

impl IntroStep {
    /// Returns the display duration for this step in seconds.
    pub fn duration_secs(&self) -> f32 {
        match self {
            IntroStep::Title => 2.5,
            IntroStep::Tagline => 2.0,
            IntroStep::TouchInstruction => 2.5,
            IntroStep::TapInstruction => 2.0,
            IntroStep::TwoFingerInstruction => 2.5,
            IntroStep::Ready => 3.0,
        }
    }

    /// Returns the next step, or None if this is the final step.
    pub fn next(&self) -> Option<IntroStep> {
        match self {
            IntroStep::Title => Some(IntroStep::Tagline),
            IntroStep::Tagline => Some(IntroStep::TouchInstruction),
            IntroStep::TouchInstruction => Some(IntroStep::TapInstruction),
            IntroStep::TapInstruction => Some(IntroStep::TwoFingerInstruction),
            IntroStep::TwoFingerInstruction => Some(IntroStep::Ready),
            IntroStep::Ready => None,
        }
    }

    /// Returns the primary text for this step.
    pub fn primary_text(&self) -> &'static str {
        match self {
            IntroStep::Title => "WHIRLED PEAS",
            IntroStep::Tagline => "A Visual Fidget Experience",
            IntroStep::TouchInstruction => "Touch & drag to paint",
            IntroStep::TapInstruction => "Tap for explosions",
            IntroStep::TwoFingerInstruction => "Two-finger tap for hyperspace",
            IntroStep::Ready => "Touch to begin",
        }
    }

    /// Returns the font size for primary text.
    pub fn primary_font_size(&self) -> f32 {
        match self {
            IntroStep::Title => 96.0,
            IntroStep::Tagline => 36.0,
            _ => 48.0,
        }
    }
}

// =============================================================================
// RESOURCES
// =============================================================================

/// Duration of fade in/out transitions in seconds.
const FADE_DURATION: f32 = 0.5;

/// Ease-in function for smooth fade-in (slow start, fast end).
fn ease_in(t: f32) -> f32 {
    t * t
}

/// Ease-out function for smooth fade-out (fast start, slow end).
fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Phase of the fade animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FadePhase {
    /// Fading in (alpha increasing)
    #[default]
    FadingIn,
    /// Fully visible
    Visible,
    /// Fading out (alpha decreasing)
    FadingOut,
}

/// Tracks the current state of the intro sequence.
#[derive(Resource)]
pub struct IntroState {
    /// Current step in the sequence
    pub current_step: IntroStep,
    /// Time elapsed in current phase
    pub phase_timer: f32,
    /// Current fade phase
    pub fade_phase: FadePhase,
    /// Whether the intro has been skipped by user touch
    pub skipped: bool,
}

impl Default for IntroState {
    fn default() -> Self {
        Self {
            current_step: IntroStep::Title,
            phase_timer: 0.0,
            fade_phase: FadePhase::FadingIn,
            skipped: false,
        }
    }
}

// =============================================================================
// STYLE CONSTANTS - BAUHAUS
// =============================================================================

/// Off-black background - deep navy void
const BG_COLOR: Color = Color::srgb(0.051, 0.051, 0.090);

/// Primary accent - vibrant spring green
const ACCENT_PRIMARY: Color = Color::srgb(0.0, 1.0, 0.533);

/// Secondary accent - chartreuse
const ACCENT_SECONDARY: Color = Color::srgb(0.498, 1.0, 0.0);

// =============================================================================
// COMPONENTS
// =============================================================================

/// Marker for the intro UI root
#[derive(Component)]
pub struct IntroUI;

/// Marker for the primary text element
#[derive(Component)]
pub struct IntroPrimaryText;

/// Marker for decorative elements
#[derive(Component)]
pub struct IntroDecor;

// =============================================================================
// SYSTEMS
// =============================================================================

/// Sets up the intro UI with Bauhaus styling.
/// Runs at Startup to ensure intro is visible before any interaction.
pub fn setup_intro_ui(mut commands: Commands, ui_font: Res<UiFont>) {
    info!(">>> INTRO: Setting up Bauhaus-styled intro UI <<<");

    let font = ui_font.handle.clone();

    // Background overlay - covers entire screen
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            ..default()
        },
        BackgroundColor(BG_COLOR),
        GlobalZIndex(150),
        IntroUI,
        IntroDecor,
        Name::new("IntroBackground"),
    ));

    // Main content container - centered flex layout
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(40.0)),
                ..default()
            },
            GlobalZIndex(200),
            IntroUI,
            Name::new("IntroContentContainer"),
        ))
        .with_children(|parent| {
            // Top decorative bar
            parent.spawn((
                Node {
                    width: Val::Px(120.0),
                    height: Val::Px(8.0),
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
                BackgroundColor(ACCENT_SECONDARY),
                IntroDecor,
                Name::new("TopBar"),
            ));

            // Primary text (starts invisible for fade-in)
            parent.spawn((
                Text::new(IntroStep::Title.primary_text()),
                TextFont {
                    font: font.clone(),
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgba(0.0, 1.0, 0.533, 0.0)), // Start transparent
                TextLayout::new_with_justify(JustifyText::Center),
                IntroPrimaryText,
                Name::new("PrimaryText"),
            ));

            // Bottom decorative bar
            parent.spawn((
                Node {
                    width: Val::Px(60.0),
                    height: Val::Px(4.0),
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                },
                BackgroundColor(ACCENT_PRIMARY),
                IntroDecor,
                Name::new("BottomBar"),
            ));
        });

    info!(">>> INTRO: UI setup complete - spawned background, bars, and text <<<");
}

/// Returns the base color for a given intro step.
fn step_color(step: IntroStep) -> Color {
    match step {
        IntroStep::Title | IntroStep::Ready => ACCENT_PRIMARY,
        IntroStep::Tagline => ACCENT_SECONDARY,
        _ => ACCENT_PRIMARY,
    }
}

/// Updates the intro sequence with fade transitions.
pub fn update_intro_sequence(
    time: Res<Time>,
    mut intro_state: ResMut<IntroState>,
    mut primary_query: Query<(&mut Text, &mut TextFont, &mut TextColor), With<IntroPrimaryText>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let dt = time.delta_secs();
    intro_state.phase_timer += dt;

    let step = intro_state.current_step;
    let visible_duration = step.duration_secs() - (2.0 * FADE_DURATION);

    match intro_state.fade_phase {
        FadePhase::FadingIn => {
            let t = (intro_state.phase_timer / FADE_DURATION).min(1.0);
            let alpha = ease_out(t); // ease-out makes fade-in more visible at start

            if let Ok((_, _, mut color)) = primary_query.get_single_mut() {
                let base = step_color(step).to_srgba();
                *color = TextColor(Color::srgba(base.red, base.green, base.blue, alpha));
            }

            if intro_state.phase_timer >= FADE_DURATION {
                intro_state.fade_phase = FadePhase::Visible;
                intro_state.phase_timer = 0.0;
            }
        }
        FadePhase::Visible => {
            if intro_state.phase_timer >= visible_duration {
                intro_state.fade_phase = FadePhase::FadingOut;
                intro_state.phase_timer = 0.0;
            }
        }
        FadePhase::FadingOut => {
            let t = (intro_state.phase_timer / FADE_DURATION).min(1.0);
            let alpha = 1.0 - ease_in(t); // ease-in makes fade-out more visible at end

            if let Ok((_, _, mut color)) = primary_query.get_single_mut() {
                let base = step_color(step).to_srgba();
                *color = TextColor(Color::srgba(base.red, base.green, base.blue, alpha));
            }

            if intro_state.phase_timer >= FADE_DURATION {
                // Transition to next step
                if let Some(next_step) = step.next() {
                    intro_state.current_step = next_step;
                    intro_state.fade_phase = FadePhase::FadingIn;
                    intro_state.phase_timer = 0.0;

                    // Update text content, font size, and reset to transparent
                    if let Ok((mut text, mut font, mut color)) = primary_query.get_single_mut() {
                        **text = next_step.primary_text().to_string();
                        font.font_size = next_step.primary_font_size();
                        // Start fully transparent for fade-in
                        let base = step_color(next_step).to_srgba();
                        *color = TextColor(Color::srgba(base.red, base.green, base.blue, 0.0));
                    }

                    info!("Intro step: {:?}", next_step);
                } else {
                    // Intro complete - transition to fidget mode
                    info!("Intro complete - transitioning to Fidget mode");
                    next_state.set(AppState::Fidget);
                }
            }
        }
    }
}

/// Handles touch/click to skip intro or proceed early on Ready step.
pub fn handle_intro_skip(
    mouse_input: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    intro_state: Res<IntroState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let touched = mouse_input.just_pressed(MouseButton::Left)
        || touches.iter_just_pressed().next().is_some();

    if touched {
        // On Ready step, any touch proceeds
        // On other steps, only proceed after fade-in is complete
        let should_proceed = match intro_state.current_step {
            IntroStep::Ready => true,
            _ => intro_state.fade_phase != FadePhase::FadingIn,
        };

        if should_proceed {
            info!("Intro skipped by user touch");
            next_state.set(AppState::Fidget);
        }
    }
}

/// Cleans up intro UI when transitioning to Fidget state.
pub fn cleanup_intro_ui(
    mut commands: Commands,
    intro_query: Query<Entity, With<IntroUI>>,
) {
    info!("Cleaning up intro UI");
    for entity in intro_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// =============================================================================
// PLUGIN
// =============================================================================

/// Plugin that manages the intro sequence.
///
/// Shows a Bauhaus-styled series of informative screens explaining
/// the app to users before transitioning to the main fidget experience.
pub struct IntroPlugin;

impl Plugin for IntroPlugin {
    fn build(&self, app: &mut App) {
        info!(">>> INTRO PLUGIN: build() starting <<<");

        app
            // Register app state
            .init_state::<AppState>()
            // Initialize intro state resource
            .init_resource::<IntroState>()
            // Setup intro UI at Startup (more reliable than OnEnter for initial state)
            .add_systems(Startup, setup_intro_ui)
            // Update systems for intro sequence
            .add_systems(
                Update,
                (update_intro_sequence, handle_intro_skip)
                    .run_if(in_state(AppState::Intro)),
            )
            // Cleanup when leaving intro state
            .add_systems(OnExit(AppState::Intro), cleanup_intro_ui);

        info!(">>> INTRO PLUGIN: build() complete <<<");
    }
}
