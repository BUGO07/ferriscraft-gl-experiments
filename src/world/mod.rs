use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use bevy_tasks::Task;
use fastnoise2::{
    SafeNode,
    generator::{Generator, GeneratorWrapper, simplex::Simplex},
};

use crate::{
    App,
    ecs::*,
    world::mesher::{Chunk, VoxelVertex},
};

pub mod generation;
pub mod mesher;

pub fn world_plugin(app: &mut App) {
    app.init_resource::<GameInfo>()
        .insert_resource(NoiseFunctions {
            seed: 1337,
            terrain: GeneratorWrapper(Simplex)
                .fbm(0.5, 0.0, 4, 2.0)
                .domain_scale(0.002)
                .build(),
            biome: GeneratorWrapper(Simplex)
                .fbm(0.6, 0.0, 3, 2.0)
                .domain_scale(0.0001)
                .build(),
            detail: GeneratorWrapper(Simplex)
                .fbm(0.5, 0.3, 3, 1.9)
                .domain_scale(0.004)
                .build(),
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
pub struct GameInfo {
    pub chunks: Arc<RwLock<HashMap<IVec3, Chunk>>>,
    pub loading_chunks: Arc<RwLock<HashSet<IVec3>>>,
}

#[derive(Resource, Clone)]
pub struct NoiseFunctions {
    pub seed: i32,
    pub terrain: GeneratorWrapper<SafeNode>,
    pub biome: GeneratorWrapper<SafeNode>,
    pub detail: GeneratorWrapper<SafeNode>,
}

#[derive(Component)]
pub struct ComputeChunk(pub Task<Chunk>, pub IVec3);

#[derive(Component)]
pub struct ComputeChunkMesh(pub Task<Option<Mesh<VoxelVertex>>>, pub IVec3);

#[derive(Component)]
pub struct ChunkMarker;
