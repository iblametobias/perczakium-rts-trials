/// Temporarely useless

impl Formation {
    const fn dimensions_as_uvec2(&self) -> UVec2 {
        uvec2(self.dimensions.0 as u32, self.dimensions.1 as u32)
    }

    /// Position relative to formation center
    fn get_desired_position(&self, member: usize) -> Vec2 {
        let pos = {
            let x = member / self.dimensions.1;
            let y = member % self.dimensions.1;
            vec2(x as f32, y as f32)
        } * self.desired_spacing + Vec2::splat(self.desired_spacing / 2.0);

        (pos - self.dimensions_rwspace() / 2.0).rotate(self.forward)
    }

    fn dimensions_rwspace(&self) -> Vec2 {
        self.dimensions_as_uvec2().as_vec2() * self.desired_spacing
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FormationMember {
    formation:  hecs::Entity,
    position:   usize
}

#[derive(Default, Debug, Clone)]
pub struct Formation {
    goal:       Vec2,
    forward:    Vec2,

    desired_velocity:       f32,
    desired_acceleration:   f32,
    desired_spacing:        f32,

    dimensions: (usize, usize),
    members:    Vec<hecs::Entity>,

    require_recompute:  bool
}
