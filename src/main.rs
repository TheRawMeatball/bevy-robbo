mod components;
mod entities;
mod game_events;
mod inventory;
mod levels;
mod plugins;
mod resources;
mod systems;

use bevy::prelude::*;
use bevy::render::render_graph::RenderGraph;
use bevy::render::renderer::{HeadlessRenderResourceContext, RenderResourceContext};
use game_events::GameEvent;
use inventory::Inventory;
use levels::{LevelInfo, LevelSet, LevelSetLoader};
use plugins::frame_cnt;
use plugins::{AudioPlugin, FrameCnt, FrameCntPlugin, KeyboardPlugin};
use resources::DamageMap;
use structopt::StructOpt;
use systems::*;

#[macro_use]
extern crate bevy_discovery;

// use std::alloc::System;
// use wasm_tracing_allocator::WasmTracingAllocator;

// #[global_allocator]
// static GLOBAL_ALLOCATOR: WasmTracingAllocator<System> = WasmTracingAllocator(System);

mod consts {
    pub const MAX_BOARD_WIDTH: i32 = 31;
    pub const MAX_BOARD_HEIGHT: i32 = 16;
    pub const STATUS_HEIGHT: i32 = 2;
}

#[derive(StructOpt, Debug, Default, Clone)]
#[structopt(name = "basic")]
pub struct Opts {
    #[structopt(short, long)]
    pub benchmark_mode: bool,

    #[structopt(short, long)]
    pub debug: bool,

    #[structopt(long)]
    pub no_audio: bool,

    #[structopt(short, long, default_value = "1")]
    pub level: usize,

    #[structopt(short, long, default_value = "8")]
    pub key_frame_interval: usize,

    #[structopt(short, long, default_value = "60")]
    pub fps: usize,

    #[structopt(long, default_value = "original.txt")]
    pub levelset_path: std::path::PathBuf,
}

pub fn render_graph_debug_system(
    render_ctx: Res<Box<dyn RenderResourceContext>>,
    render_graph: Res<RenderGraph>,
) {
    let _ = render_ctx
        .as_any()
        .downcast_ref::<HeadlessRenderResourceContext>();
    info!("render graph: {:?}", *render_graph);
}

pub fn debug_system(shaders: Res<Assets<Shader>>, render_graph: Res<RenderGraph>) {
    info!("num shaders: {}", &shaders.iter().count());
    info!("render_graph: {:#?}", *render_graph);
}

fn main() {
    let opts = Opts::from_args();
    info!("opts: {:?}", opts);

    let vsync = opts.fps == 60 && !opts.benchmark_mode;
    let mut builder = App::build();

    builder.add_plugin(plugins::RenderPlugin { vsync });
    builder
        .add_resource(WindowDescriptor {
            title: "Robbo".to_string(),
            width: (32 * consts::MAX_BOARD_WIDTH) as f32,
            height: (32 * (consts::MAX_BOARD_HEIGHT + consts::STATUS_HEIGHT)) as f32,
            resizable: true,
            // mode: window::WindowMode::Fullscreen {use_size: false},
            mode: bevy::window::WindowMode::Windowed,
            #[cfg(target_arch = "wasm32")]
            canvas: Some("#bevy-canvas".to_string()),
            vsync: vsync,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins);

    #[cfg(target_arch = "wasm32")]
    builder.add_plugin(bevy_webgl2::WebGL2Plugin::default());

    builder
        .add_resource(Inventory::default())
        .add_resource(LevelInfo::default())
        .add_resource(DamageMap::default())
        .add_resource(Events::<GameEvent>::default())
        .add_resource(opts.clone())
        .add_asset::<LevelSet>()
        .init_asset_loader::<LevelSetLoader>()
        .add_plugin(FrameCntPlugin::new(opts.key_frame_interval))
        .add_plugin(KeyboardPlugin)
        .add_plugin(AudioPlugin)
        .add_stage_before(stage::UPDATE, "move", SystemStage::parallel())
        .add_stage_before(stage::UPDATE, "move_robbo", SystemStage::parallel())
        .add_stage_before(stage::POST_UPDATE, "reload_level", SystemStage::parallel())
        .add_stage_before(stage::POST_UPDATE, "shots", SystemStage::parallel())
        .add_stage_before(
            stage::POST_UPDATE,
            "process_damage",
            SystemStage::parallel(),
        )
        .add_stage_before(stage::POST_UPDATE, "game_events", SystemStage::parallel())
        .add_stage_after("keyboard", "magnetic_field", SystemStage::parallel())
        .add_stage_after("frame_cnt", "tick", SystemStage::parallel())
        .add_plugin(DiscoveryPlugin)
        .add_startup_system(level_setup.system())
        .add_system_to_stage(stage::EVENT, update_game_events.system())
        .add_system_to_stage(stage::EVENT, asset_events.system());
    if opts.debug {
        builder.add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
        //.add_plugin(bevy::diagnostic::PrintDiagnosticsPlugin::default());
    }

    if !opts.benchmark_mode {
        builder.add_system_to_stage("reload_level", reload_level.system());
        if !vsync {
            #[cfg(not(target_arch = "wasm32"))]
            builder.add_plugin(plugins::FrameLimiterPlugin {
                fps: opts.fps as f32,
            });
        }
    } else {
        builder.add_system_to_stage("reload_level", benchmark_reload_level.system());
    }
    builder.run();
}

#[derive(DiscoveryPlugin)]
#[root("src/main.rs")]
struct DiscoveryPlugin;