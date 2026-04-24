use winit::{
    event::KeyEvent,
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
};

use crate::{Direction, State, TRANSLATION_SPEED};

pub fn handle_input(event_loop: &ActiveEventLoop, event: &KeyEvent, state: &mut State) {
    if event.logical_key == Key::Named(NamedKey::Escape) {
        event_loop.exit();
    }

    handle_translation(event, state);
    handle_rotation(event, state);
}

fn handle_translation(event: &KeyEvent, state: &mut State) {
    handle_direction(
        event,
        Key::Named(NamedKey::ArrowLeft),
        Direction::Dec,
        &mut state.translation.x_direction,
    );
    handle_direction(
        event,
        Key::Named(NamedKey::ArrowRight),
        Direction::Inc,
        &mut state.translation.x_direction,
    );
    handle_direction(
        event,
        Key::Named(NamedKey::ArrowDown),
        Direction::Inc,
        &mut state.translation.y_direction,
    );
    handle_direction(
        event,
        Key::Named(NamedKey::ArrowUp),
        Direction::Dec,
        &mut state.translation.y_direction,
    );

    if state.translation.x_direction == Direction::Dec {
        state.translation.x_speed = -TRANSLATION_SPEED;
    }
    if state.translation.x_direction == Direction::Inc {
        state.translation.x_speed = TRANSLATION_SPEED;
    }
    if state.translation.x_direction == Direction::None {
        state.translation.x_speed = 0.;
    }

    if state.translation.y_direction == Direction::Dec {
        state.translation.y_speed = -TRANSLATION_SPEED;
    }
    if state.translation.y_direction == Direction::Inc {
        state.translation.y_speed = TRANSLATION_SPEED;
    }
    if state.translation.y_direction == Direction::None {
        state.translation.y_speed = 0.;
    }
}

fn handle_rotation(event: &KeyEvent, state: &mut State) {
    handle_direction(
        event,
        Key::Character("q".into()),
        Direction::Inc,
        &mut state.rotation.direction,
    );
    handle_direction(
        event,
        Key::Character("f".into()),
        Direction::Dec,
        &mut state.rotation.direction,
    );
}

fn handle_direction(
    event: &KeyEvent,
    logical_key: Key,
    pressed_direction: Direction,
    value_ref: &mut Direction,
) {
    if event.logical_key == logical_key {
        if event.state.is_pressed() {
            *value_ref = pressed_direction;
        } else if *value_ref == pressed_direction {
            *value_ref = Direction::None;
        }
    }
}
