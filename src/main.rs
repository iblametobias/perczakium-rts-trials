use macroquad::{color::colors, input, math::vec2, rand};
use perczakium_rts_trials::{game::{Game, TeamMember}, update_manager::UpdateManager};

#[macroquad::main("RTS TEST")]
async fn main() {
    input::prevent_quit();

    let map_border = vec2(1024.0, 1024.0);
    let mut game = Game::new(map_border);
    game.formation_spacing_multiplier = 2.0;

    let team_a = game.spawn_team(colors::RED);
    let team_b = game.spawn_team(colors::BLUE);

    let mut units_a = vec![];
    let mut units_b = vec![];

    let mut update_manager = UpdateManager::new(20.0);
    let mut time_speed_multiplier = 1f32;

    let rand_position = || vec2(
        rand::gen_range(0.0, map_border.x),
        rand::gen_range(0.0, map_border.y),
    );

    for _ in 0..64 {
        let u = game.spawn_unit(rand_position(), TeamMember(team_a));
        units_a.push(u);
    }
    for _ in 0..64 {
        let u = game.spawn_unit(rand_position(), TeamMember(team_b));
        units_b.push(u);
    }

    loop {
        if input::is_quit_requested() {
            let perf = update_manager.take_profiler().get_stats();
            eprintln!("{}", perf);
            break;
        }

        update_manager.update(|dt| {
            game.update(dt.mul_f32(time_speed_multiplier));
        });
        update_manager.draw_profiler();

        if input::is_mouse_button_pressed(input::MouseButton::Right) {
            game.set_task(
                input::mouse_position().into()
            );
        }
        if input::is_mouse_button_pressed(input::MouseButton::Left) {
            game.unset_task();
        }
        if input::is_key_pressed(input::KeyCode::Equal) {
            time_speed_multiplier *= 2.0;
            eprintln!("Simulation speed: {time_speed_multiplier:.2}x");
        }
        if input::is_key_pressed(input::KeyCode::Minus) {
            time_speed_multiplier *= 0.5;
            eprintln!("Simulation speed: {time_speed_multiplier:.2}x");
        }
        if input::is_key_pressed(input::KeyCode::PageUp) {
            game.formation_spacing_multiplier *= 1.5;
            eprintln!("Formation spacing: {:.2}*unit radius", game.formation_spacing_multiplier);
        }
        if input::is_key_pressed(input::KeyCode::PageDown) {
            game.formation_spacing_multiplier /= 1.5;
            eprintln!("Formation spacing: {:.2}*unit radius", game.formation_spacing_multiplier);
        }



        game.render();

        macroquad::window::next_frame().await
    }
}
