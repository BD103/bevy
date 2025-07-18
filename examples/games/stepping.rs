use bevy::{app::MainScheduleOrder, ecs::schedule::*, prelude::*};

/// Independent [`Schedule`] for stepping systems.
///
/// The stepping systems must run in their own schedule to be able to inspect
/// all the other schedules in the [`App`].  This is because the currently
/// executing schedule is removed from the [`Schedules`] resource while it is
/// being run.
#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
struct DebugSchedule;

/// Plugin to add a stepping UI to an example
#[derive(Default)]
pub struct SteppingPlugin {
    schedule_labels: Vec<InternedScheduleLabel>,
    top: Val,
    left: Val,
}

impl SteppingPlugin {
    /// add a schedule to be stepped when stepping is enabled
    pub fn add_schedule(mut self, label: impl ScheduleLabel) -> SteppingPlugin {
        self.schedule_labels.push(label.intern());
        self
    }

    /// Set the location of the stepping UI when activated
    pub fn at(self, left: Val, top: Val) -> SteppingPlugin {
        SteppingPlugin { top, left, ..self }
    }
}

impl Plugin for SteppingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_stepping_hint);
        if cfg!(not(feature = "bevy_debug_stepping")) {
            return;
        }

        // create and insert our debug schedule into the main schedule order.
        // We need an independent schedule so we have access to all other
        // schedules through the `Stepping` resource
        app.init_schedule(DebugSchedule);
        let mut order = app.world_mut().resource_mut::<MainScheduleOrder>();
        order.insert_after(Update, DebugSchedule);

        // create our stepping resource
        let mut stepping = Stepping::new();
        for label in &self.schedule_labels {
            stepping.add_schedule(*label);
        }
        app.insert_resource(stepping);

        // add our startup & stepping systems
        app.insert_resource(State {
            ui_top: self.top,
            ui_left: self.left,
            systems: Vec::new(),
        })
        .add_systems(
            DebugSchedule,
            (
                build_ui.run_if(not(initialized)),
                handle_input,
                update_ui.run_if(initialized),
            )
                .chain(),
        );
    }
}

/// Struct for maintaining stepping state
#[derive(Resource, Debug)]
struct State {
    // vector of schedule/node id -> text index offset
    systems: Vec<(InternedScheduleLabel, NodeId, usize)>,

    // ui positioning
    ui_top: Val,
    ui_left: Val,
}

/// condition to check if the stepping UI has been constructed
fn initialized(state: Res<State>) -> bool {
    !state.systems.is_empty()
}

const FONT_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);
const FONT_BOLD: &str = "fonts/FiraSans-Bold.ttf";

#[derive(Component)]
struct SteppingUi;

/// Construct the stepping UI elements from the [`Schedules`] resource.
///
/// This system may run multiple times before constructing the UI as all of the
/// data may not be available on the first run of the system.  This happens if
/// one of the stepping schedules has not yet been run.
fn build_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    schedules: Res<Schedules>,
    mut stepping: ResMut<Stepping>,
    mut state: ResMut<State>,
) {
    let mut text_spans = Vec::new();
    let mut always_run: Vec<(
        bevy_ecs::intern::Interned<dyn ScheduleLabel + 'static>,
        NodeId,
    )> = Vec::new();

    let Ok(schedule_order) = stepping.schedules() else {
        return;
    };

    // go through the stepping schedules and construct a list of systems for
    // each label
    for label in schedule_order {
        let schedule = schedules.get(*label).unwrap();
        text_spans.push((
            TextSpan(format!("{label:?}\n")),
            TextFont {
                font: asset_server.load(FONT_BOLD),
                ..default()
            },
            TextColor(FONT_COLOR),
        ));

        // grab the list of systems in the schedule, in the order the
        // single-threaded executor would run them.
        let Ok(systems) = schedule.systems() else {
            return;
        };

        for (key, system) in systems {
            // skip bevy default systems; we don't want to step those
            #[cfg(feature = "debug")]
            if system.name().as_string().starts_with("bevy") {
                always_run.push((*label, NodeId::System(key)));
                continue;
            }

            // Add an entry to our systems list so we can find where to draw
            // the cursor when the stepping cursor is at this system
            // we add plus 1 to account for the empty root span
            state
                .systems
                .push((*label, NodeId::System(key), text_spans.len() + 1));

            // Add a text section for displaying the cursor for this system
            text_spans.push((
                TextSpan::new("   "),
                TextFont::default(),
                TextColor(FONT_COLOR),
            ));

            // add the name of the system to the ui
            text_spans.push((
                TextSpan(format!("{}\n", system.name())),
                TextFont::default(),
                TextColor(FONT_COLOR),
            ));
        }
    }

    for (label, node) in always_run.drain(..) {
        stepping.always_run_node(label, node);
    }

    commands.spawn((
        Text::default(),
        SteppingUi,
        Node {
            position_type: PositionType::Absolute,
            top: state.ui_top,
            left: state.ui_left,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.33)),
        Visibility::Hidden,
        Children::spawn(text_spans),
    ));
}

fn build_stepping_hint(mut commands: Commands) {
    let hint_text = if cfg!(feature = "bevy_debug_stepping") {
        "Press ` to toggle stepping mode (S: step system, Space: step frame)"
    } else {
        "Bevy was compiled without stepping support. Run with `--features=bevy_debug_stepping` to enable stepping."
    };
    info!("{}", hint_text);
    // stepping description box
    commands.spawn((
        Text::new(hint_text),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(FONT_COLOR),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
    ));
}

fn handle_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut stepping: ResMut<Stepping>) {
    if keyboard_input.just_pressed(KeyCode::Slash) {
        info!("{:#?}", stepping);
    }
    // grave key to toggle stepping mode for the FixedUpdate schedule
    if keyboard_input.just_pressed(KeyCode::Backquote) {
        if stepping.is_enabled() {
            stepping.disable();
            debug!("disabled stepping");
        } else {
            stepping.enable();
            debug!("enabled stepping");
        }
    }

    if !stepping.is_enabled() {
        return;
    }

    // space key will step the remainder of this frame
    if keyboard_input.just_pressed(KeyCode::Space) {
        debug!("continue");
        stepping.continue_frame();
    } else if keyboard_input.just_pressed(KeyCode::KeyS) {
        debug!("stepping frame");
        stepping.step_frame();
    }
}

fn update_ui(
    mut commands: Commands,
    state: Res<State>,
    stepping: Res<Stepping>,
    ui: Single<(Entity, &Visibility), With<SteppingUi>>,
    mut writer: TextUiWriter,
) {
    // ensure the UI is only visible when stepping is enabled
    let (ui, vis) = *ui;
    match (vis, stepping.is_enabled()) {
        (Visibility::Hidden, true) => {
            commands.entity(ui).insert(Visibility::Inherited);
        }
        (Visibility::Hidden, false) | (_, true) => (),
        (_, false) => {
            commands.entity(ui).insert(Visibility::Hidden);
        }
    }

    // if we're not stepping, there's nothing more to be done here.
    if !stepping.is_enabled() {
        return;
    }

    let (cursor_schedule, cursor_system) = match stepping.cursor() {
        // no cursor means stepping isn't enabled, so we're done here
        None => return,
        Some(c) => c,
    };

    for (schedule, system, text_index) in &state.systems {
        let mark = if &cursor_schedule == schedule && *system == cursor_system {
            "-> "
        } else {
            "   "
        };
        *writer.text(ui, *text_index) = mark.to_string();
    }
}
