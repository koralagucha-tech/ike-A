use {
    smash::{
        lua2cpp::*,
        phx::*,
        app::{sv_animcmd::*, lua_bind::*, *},
        lib::{lua_const::*, L2CValue, L2CAgent},
        hash40
    },
    smash_script::*,
    smashline::{*, Priority::*}
};

// Down taunt: show the "hanx123" mesh and hide the "shirt" mesh.
unsafe extern "C" fn ike_game_appeallw(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 20.0);
    if macros::is_excute(agent) {
        ModelModule::set_mesh_visibility(agent.module_accessor, Hash40::new("hanx123"), true);
        ModelModule::set_mesh_visibility(agent.module_accessor, Hash40::new("shirt"), false);
    }
}

pub fn install() {
    Agent::new("ike")
        .game_acmd("game_appeallwl", ike_game_appeallw, Default)
        .game_acmd("game_appeallwr", ike_game_appeallw, Default)
        .install();
}
