use std::time::Duration;

use macroquad::{color::{self, colors}, math::{Vec2}, shapes};

const SMALL_F32: f32 = 1e-2;

#[derive(Default)]
pub struct Game {
    entities:       hecs::World,
    units_dest:     Option<Vec2>,
    map_border:     Vec2,
    update_number:  u32,
    pub formation_spacing_multiplier: f32,
}

impl Game {
    pub fn new(map_border: Vec2) -> Self {
        Self {
            map_border,
            ..Default::default()
        }
    }

    pub fn update(&mut self, delta_time: Duration) {
        let dt = delta_time.as_secs_f32();

        for (pos, prev_pos, vel, prev_vel) in self.entities
            .query_mut::<(&Position, &mut PreviousPosition, &Velocity, &mut PreviousVelocity)>() {
            prev_pos.0 = pos.0;
            prev_vel.0 = vel.0;
        }

        // FORMATIONLESS UNITS

        let avoidance_prediction_time = 6f32;
        let avoidance_margin = 4f32;
        let avoidance_view_range = 8f32;
        let avoidance_priority = 3f32;

        for (this, pos, vel, f, col, spec) in self.entities
            .query::<(hecs::Entity, &Position, &Velocity, &mut Forces, &Collider, &MovementForce)>().iter() {
            let mut avoidance = Vec2::ZERO;

            for (other, pos_other, vel_other, col_other) in self.entities
                .query::<(hecs::Entity, &PreviousPosition, &PreviousVelocity, &Collider)>().iter() {
                if this == other { continue; }

                let rel_pos = pos_other.0 - pos.0;

                if rel_pos.length_squared() > avoidance_view_range.powi(2) { continue; }

                let rel_vel = vel_other.0 - vel.0;
                let rel_speed_sq = rel_vel.length_squared();

                let t = if rel_speed_sq > SMALL_F32 {
                    (-rel_pos.dot(rel_vel) / rel_speed_sq)
                    .clamp(0.0, avoidance_prediction_time)
                } else { 0.0 };

                let safe_distance = col.radius + col_other.radius + avoidance_margin;

                let future_offset = rel_pos + rel_vel * t;
                let future_dist = future_offset.length();

                if future_dist < safe_distance {
                    let strength = 1.0 - future_dist / safe_distance;
                    avoidance += -future_offset.normalize_or_zero() * strength;
                }
            }

            avoidance = avoidance.clamp_length_max(1.0);

            let task_direction = match self.units_dest {
                Some(d) => (d - pos.0).clamp_length_max(1.0),
                None => Vec2::ZERO,
            };

            let movement = (task_direction + avoidance * avoidance_priority).clamp_length_max(1.0);

            f.0 += movement * spec.force;
        }

        let resistance = 1.0 - GLOBAL_RESISTANCE.powf(dt);

        for (pos, v, f, m) in self.entities
            .query_mut::<(&mut Position, &mut Velocity, &mut Forces, &Mass)>() {
            v.0 += f.0 / m.0 * dt;
            f.0 = Vec2::ZERO;
            v.0 *= 1.0 - resistance;
            pos.0 += v.0 * dt;
        }

        for (pos, col) in self.entities
            .query_mut::<(&mut Position, &Collider)>() {
            let rr = Vec2::splat(col.radius);
            pos.0 = pos.0.clamp(rr, self.map_border - rr);
        }

        for _ in 0..2 {
            self.resolve_collisions();
        }

        self.update_number += 1;
    }

    pub fn render(&self) {
        for (&Position(Vec2 { x, y }), &Collider { radius }) in self
            .entities
            .query::<(&Position, &Collider)>()
            .without::<&TeamMember>()
            .iter()
        {
            shapes::draw_circle(x, y, radius, colors::WHITE);
        }

        for (&Position(Vec2 { x, y }), &Collider { radius }, team) in self
            .entities
            .query::<(&Position, &Collider, &TeamMember)>()
            .iter()
        {
            let color = self
                .entities
                .get::<&TeamColor>(team.0).unwrap().0;

            shapes::draw_circle(x, y, radius, color);
        }
    }

    pub fn spawn_team(&mut self, color: color::Color) -> hecs::Entity {
        self.entities.spawn((TeamColor(color), ))
    }

    pub fn spawn_unit(&mut self, pos: Vec2, team: TeamMember) -> hecs::Entity {
        self.entities.spawn(test_unit_builder(pos, team).build())
    }

    pub fn set_task(&mut self, destination: Vec2) {
        self.units_dest = Some(destination);
    }

    pub fn unset_task(&mut self) {
        self.units_dest = None;
    }

    fn resolve_collisions(&mut self) {
        // TODO: Use mass in collisions
        for (this, Position(pos), Velocity(vel), col)
        in self.entities.query::<(hecs::Entity, &mut Position, &mut Velocity, &Collider)>().iter() {
            let mut push = Vec2::ZERO;
            let mut contacts = 0u32;

            for (other, &PreviousPosition(pos_other), col_other)
            in self.entities.query::<(hecs::Entity, &PreviousPosition, &Collider)>().iter() {
                if this == other { continue; }

                let min_distance = col.radius + col_other.radius;

                let delta = *pos - pos_other;
                let distance = delta.length();
                let direction = delta.normalize_or(
                    if this > other { Vec2::X } else { Vec2::NEG_X }
                );

                let penetration = min_distance - distance;

                if penetration <= 0.0 { continue; }

                let vn = vel.dot(direction);
                if vn < 0.0 {
                    *vel -= direction * vn;
                }
                let displacement = penetration * direction / 2.0;
                push += displacement;
                contacts += 1;
            }

            if contacts > 0 {
                *pos += push / contacts as f32;
            }
        }
    }



    /// True once every 5 updates
    fn sparse_update(&self) -> bool {
        self.update_number % 5 == 0
    }

    /// True once every 20 updates
    fn sparsererer_update(&self) -> bool {
        self.update_number % 20 == 0
    }
}

#[derive(hecs::Bundle)]
pub struct KineticBody {
    pub c_position: Position,
    pub p_position: PreviousPosition,
    pub c_velocity: Velocity,
    pub p_velocity: PreviousVelocity,
    pub forces:     Forces,
    pub mass:       Mass,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Position(pub Vec2);
#[derive(Default, Debug, Clone, Copy)]
pub struct PreviousPosition(pub Vec2);

#[derive(Default, Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);
#[derive(Default, Debug, Clone, Copy)]
pub struct PreviousVelocity(pub Vec2);

#[derive(Default, Debug, Clone, Copy)]
pub struct Forces(pub Vec2);

#[derive(Default, Debug, Clone, Copy)]
pub struct Mass(pub f32);

#[derive(Debug, Clone, Copy)]
pub struct TeamMember(pub hecs::Entity);

#[derive(Debug, Clone, Copy)]
pub struct TeamColor(pub color::Color);

#[derive(Default)]
pub struct Collider {
    radius: f32
}

pub struct MovementForce {
    force:  f32,
}

impl MovementForce {
    /// Temporary vibe coded version
    fn new(
        mass: Mass,             // mu
        speed: f32,             // u/second
        acceleration_time: f32, // seconds
        resistance: f32,        // fraction of velocity kept after 1 second
    ) -> Self {
        let m = mass.0; // adjust to however Mass exposes its f32
        let t = acceleration_time;

        // drag: v' = F/m - k*v, with e^-k = resistance
        let k = -resistance.clamp(1e-6, 1.0).ln();

        // v(t) = F/(m*k) * (1 - e^(-k*t)) = F/(m*k) * (1 - resistance^t)
        // solve v(acceleration_time) = speed for F
        let force = if k < 1e-6 {
            m * speed / t // no drag limit: F = m*v/t
        } else {
            m * k * speed / (1.0 - (-k * t).exp())
        };

        Self { force }
    }
}

/// Fraction of velocity that remains after 1 second
const GLOBAL_RESISTANCE: f32 = 0.1;

const TEST_UNIT_MASS: Mass = Mass(1.0);
const TEST_UNIT_SPEED: f32 = 30.0;
const TEST_UNIT_ACCELERATION_TIME: f32 = 1.0;
const TEST_UNIT_COLLIDER: Collider = Collider { radius: 2.71 };

fn test_unit_builder(pos: Vec2, team: TeamMember) -> hecs::EntityBuilder {
    let mut builder = hecs::EntityBuilder::new();

    builder.add_bundle(
        KineticBody {
            c_position: Position(pos),
            p_position: PreviousPosition(pos),
            c_velocity: Velocity::default(),
            p_velocity: PreviousVelocity::default(),
            mass:       TEST_UNIT_MASS,
            forces:     Forces::default()
        }
    );
    builder.add(TEST_UNIT_COLLIDER);
    builder.add(MovementForce::new(
        TEST_UNIT_MASS,
        TEST_UNIT_SPEED,
        TEST_UNIT_ACCELERATION_TIME,
        GLOBAL_RESISTANCE
    ));
    builder.add(team);

    builder
}
