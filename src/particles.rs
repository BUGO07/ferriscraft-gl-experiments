use crate::{App, ecs::*};

#[derive(PartialEq, Clone, Copy)]
pub struct Particle {
    pub transform: Transform,
    pub velocity: Vec3,
    pub life: f32,
    pub color: Vec4,
}

#[derive(Component)]
pub struct ParticleEmitter {
    pub particles: Vec<Particle>,
}

pub fn particle_plugin(app: &mut App) {
    app.add_systems(Update, update_particles);
}

fn update_particles(mut emitters: Query<&mut ParticleEmitter>, time: Res<Time>) {
    for mut emitter in emitters.iter_mut() {
        let mut new_particles = Vec::new();
        for particle in emitter.particles.iter_mut() {
            particle.life -= time.delta_secs();
            if particle.life > 0.0 {
                particle.transform.translation += particle.velocity;
                new_particles.push(*particle);
            }
        }
        emitter.particles = new_particles;
    }
}
