use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use bevy_tasks::Task;
use noise::{Fbm, MultiFractal, Simplex};

use crate::{
    App,
    ecs::*,
    world::mesher::{Chunk, ChunkMesh},
};

pub mod generation;
pub mod interaction;
pub mod mesher;

pub fn world_plugin(app: &mut App) {
    let seed = 1337;
    app.init_resource::<WorldData>()
        .insert_resource(NoiseFunctions {
            seed,
            terrain: Fbm::<Simplex>::new(seed)
                .set_frequency(0.002)
                .set_persistence(0.5)
                .set_octaves(4)
                .set_lacunarity(2.0),
            biome: Fbm::<Simplex>::new(seed + 1)
                .set_frequency(0.0001)
                .set_persistence(0.6)
                .set_octaves(3)
                .set_lacunarity(2.0),
            // detail: Fbm::<Simplex>::new(seed)
            //     .set_frequency(0.004)
            //     .set_persistence(0.5)
            //     .set_octaves(3)
            //     .set_lacunarity(1.9),
        })
        .add_systems(
            Update,
            (
                generation::handle_chunk_gen,
                generation::handle_mesh_gen,
                generation::handle_chunk_despawn,
                generation::process_tasks,
            ),
        );
}

#[derive(Resource, Default)]
pub struct WorldData {
    pub chunks: Arc<RwLock<HashMap<IVec3, Chunk>>>,
    pub loading_chunks: Arc<RwLock<HashSet<IVec3>>>,
}

#[derive(Resource, Clone)]
pub struct NoiseFunctions {
    pub seed: u32,
    pub terrain: Fbm<Simplex>,
    pub biome: Fbm<Simplex>,
    // pub detail: Fbm<Simplex>,
}

#[derive(Component)]
pub struct ComputeChunk(pub Task<Chunk>, pub IVec3);

#[derive(Component)]
pub struct ComputeChunkMesh(pub Task<Option<ChunkMesh>>, pub IVec3);

#[derive(Component)]
pub struct ChunkMarker;
