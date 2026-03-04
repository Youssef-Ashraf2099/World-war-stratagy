use bevy::prelude::*;

/// Current speed setting — the number of in-game ticks per real second.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameSpeed {
    Speed1, // 1 tick / sec
    Speed2, // 2 ticks / sec
    Speed3, // 4 ticks / sec
}

impl GameSpeed {
    pub fn tick_interval_secs(self) -> f32 {
        match self {
            GameSpeed::Speed1 => 1.0,
            GameSpeed::Speed2 => 0.5,
            GameSpeed::Speed3 => 0.25,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            GameSpeed::Speed1 => "1×",
            GameSpeed::Speed2 => "2×",
            GameSpeed::Speed3 => "4×",
        }
    }
}

/// Local clock state — tracks paused, speed, tick counter and the timer.
#[derive(Resource)]
pub struct GameClock {
    pub paused: bool,
    pub speed: GameSpeed,
    pub tick: u64,
    /// Simulated year/month counters (starts January 1939 by default).
    pub year: i32,
    pub month: u8,
    /// Internal timer used to gate tick advancement.
    pub timer: Timer,
}

impl Default for GameClock {
    fn default() -> Self {
        let speed = GameSpeed::Speed1;
        Self {
            paused: true, // start paused so player can see the map before action begins
            speed,
            tick: 0,
            year: 1939,
            month: 1,
            timer: Timer::from_seconds(speed.tick_interval_secs(), TimerMode::Repeating),
        }
    }
}

impl GameClock {
    /// Returns the formatted date string shown in the HUD top-right.
    pub fn date_string(&self) -> String {
        const MONTHS: &[&str] = &[
            "Jan", "Feb", "Mar", "Apr", "May", "Jun",
            "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let month_name = MONTHS[(self.month as usize).saturating_sub(1).min(11)];
        format!("{} {} | Turn {}", month_name, self.year, self.tick)
    }

    /// Advance the simulated calendar by one month.
    fn advance_month(&mut self) {
        self.month += 1;
        if self.month > 12 {
            self.month = 1;
            self.year += 1;
        }
        self.tick += 1;
    }

    /// Change speed, resetting the tick timer to the new interval.
    pub fn set_speed(&mut self, speed: GameSpeed) {
        self.speed = speed;
        self.timer
            .set_duration(std::time::Duration::from_secs_f32(speed.tick_interval_secs()));
        self.timer.reset();
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Advance the local game clock when not paused.
pub fn advance_clock(time: Res<Time>, mut clock: ResMut<GameClock>) {
    if clock.paused {
        return;
    }
    clock.timer.tick(time.delta());
    if clock.timer.just_finished() {
        clock.advance_month();
    }
}

/// Keyboard shortcuts to control the clock:
///   Space — toggle pause/play
///   1 / 2 / 3 — set speed
pub fn clock_keyboard_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut clock: ResMut<GameClock>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        clock.paused = !clock.paused;
        info!(
            "Game clock {}",
            if clock.paused { "PAUSED" } else { "RUNNING" }
        );
    }
    if keyboard.just_pressed(KeyCode::Digit1) {
        clock.set_speed(GameSpeed::Speed1);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        clock.set_speed(GameSpeed::Speed2);
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        clock.set_speed(GameSpeed::Speed3);
    }
}

/// Marker for the HUD date label so `update_date_label` can find it.
#[derive(Component)]
pub struct DateLabel;

/// Update the top-right date text from the current clock state.
pub fn update_date_label(
    clock: Res<GameClock>,
    mut query: Query<&mut Text, With<DateLabel>>,
) {
    if !clock.is_changed() {
        return;
    }
    for mut text in &mut query {
        if let Some(section) = text.sections.first_mut() {
            section.value = clock.date_string();
        }
    }
}
