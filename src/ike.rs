use {
    smash::{
        lua2cpp::*,
        phx::*,
        app::{sv_animcmd::*, lua_bind::*, *},
        lib::lua_const::*,
    },
    smash_script::*,
    smashline::{*, Priority::*},
    std::sync::atomic::{AtomicU32, Ordering},
};

// One bit per player slot (entry id 0..7).
// bit = 0 -> default state: `hanx123` hidden, `shirt` visible
// bit = 1 -> toggled state: `hanx123` visible, `shirt` hidden
static TOGGLED: AtomicU32 = AtomicU32::new(0);

unsafe fn entry_bit(module_accessor: *mut BattleObjectModuleAccessor) -> u32 {
    let entry_id = WorkModule::get_int(module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    1u32 << (entry_id & 7)
}

unsafe fn apply_meshes(module_accessor: *mut BattleObjectModuleAccessor) {
    let toggled = TOGGLED.load(Ordering::Relaxed) & entry_bit(module_accessor) != 0;
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("hanx123"), toggled);
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("shirt"), !toggled);
}

// Runs once when the fighter is created at the start of a match: back to the default state.
unsafe extern "C" fn ike_start(fighter: &mut L2CFighterCommon) {
    let module_accessor = fighter.module_accessor;
    TOGGLED.fetch_and(!entry_bit(module_accessor), Ordering::Relaxed);
    apply_meshes(module_accessor);
}

// Runs every frame: keeps the meshes in the right state (this is what hides `hanx123`
// by default, and keeps it that way after respawns, hitstun, etc.).
unsafe extern "C" fn ike_frame(fighter: &mut L2CFighterCommon) {
    apply_meshes(fighter.module_accessor);
}

// Down taunt: flip between the default state and the toggled state.
unsafe extern "C" fn ike_game_appeallw(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    if macros::is_excute(agent) {
        TOGGLED.fetch_xor(entry_bit(agent.module_accessor), Ordering::Relaxed);
        apply_meshes(agent.module_accessor);
    }
}

pub fn install() {
    Agent::new("ike")
        .on_start(ike_start)
        .on_line(Main, ike_frame)
        .game_acmd("game_appeallwl", ike_game_appeallw, Default)
        .game_acmd("game_appeallwr", ike_game_appeallw, Default)
        .install();
}
