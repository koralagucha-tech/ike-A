use {
    smash::{
        lua2cpp::*,
        app::{lua_bind::*, *},
        lib::lua_const::*,
        hash40,
    },
    smashline::*,
    std::sync::atomic::{AtomicU32, Ordering},
};

// One bit per player slot (entry id 0..7).
// TOGGLED bit = 0 -> default state: `hanx123` hidden, `shirt` visible
// TOGGLED bit = 1 -> toggled state: `hanx123` visible, `shirt` hidden
static TOGGLED: AtomicU32 = AtomicU32::new(0);
// Bit = 1 while that player is already inside the down taunt (so we toggle only once per taunt).
static IN_TAUNT: AtomicU32 = AtomicU32::new(0);

unsafe fn entry_bit(module_accessor: *mut BattleObjectModuleAccessor) -> u32 {
    let entry_id = WorkModule::get_int(module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    1u32 << (entry_id & 7)
}

unsafe fn apply_meshes(module_accessor: *mut BattleObjectModuleAccessor) {
    let toggled = TOGGLED.load(Ordering::Relaxed) & entry_bit(module_accessor) != 0;
    ModelModule::set_mesh_visibility(module_accessor, smash::phx::Hash40::new("hanx123"), toggled);
    ModelModule::set_mesh_visibility(module_accessor, smash::phx::Hash40::new("shirt"), !toggled);
}

// Runs once when the fighter is created at the start of a match: back to the default state.
unsafe extern "C" fn ike_start(fighter: &mut L2CFighterCommon) {
    let module_accessor = fighter.module_accessor;
    let bit = entry_bit(module_accessor);
    TOGGLED.fetch_and(!bit, Ordering::Relaxed);
    IN_TAUNT.fetch_and(!bit, Ordering::Relaxed);
    apply_meshes(module_accessor);
}

// Runs every frame. Instead of replacing Ike's taunt script (which was making the sword
// disappear), we leave the vanilla taunt untouched and just watch for the down taunt motion.
unsafe extern "C" fn ike_frame(fighter: &mut L2CFighterCommon) {
    let module_accessor = fighter.module_accessor;
    let bit = entry_bit(module_accessor);

    let motion = MotionModule::motion_kind(module_accessor);
    let in_down_taunt = motion == hash40("appeal_lw_l") || motion == hash40("appeal_lw_r");

    if in_down_taunt {
        // First frame of the taunt: flip between default and toggled state.
        if IN_TAUNT.fetch_or(bit, Ordering::Relaxed) & bit == 0 {
            TOGGLED.fetch_xor(bit, Ordering::Relaxed);
        }
    } else {
        IN_TAUNT.fetch_and(!bit, Ordering::Relaxed);
    }

    // Keep the meshes in the right state every frame (this is what hides `hanx123` by default).
    apply_meshes(module_accessor);
}

pub fn install() {
    Agent::new("ike")
        .on_start(ike_start)
        .on_line(Main, ike_frame)
        .install();
}
