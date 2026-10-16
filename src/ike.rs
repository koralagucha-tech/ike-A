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

// One bit per player slot (entry id 0..7) in each of these.
//
// DOWN taunt toggle:  bit 0 = default (`hanx123` hidden, `shirt` visible)
//                     bit 1 = toggled (`hanx123` visible, `shirt` hidden)
// SIDE taunt toggle:  bit 0 = default (`mask123` visible)
//                     bit 1 = toggled (`mask123` hidden)   <- same setting as `shirt`
static DOWN_TOGGLED: AtomicU32 = AtomicU32::new(0);
static SIDE_TOGGLED: AtomicU32 = AtomicU32::new(0);

// Bit = 1 while that player is already inside the taunt (so we toggle only once per taunt).
static IN_DOWN_TAUNT: AtomicU32 = AtomicU32::new(0);
static IN_SIDE_TAUNT: AtomicU32 = AtomicU32::new(0);

unsafe fn entry_bit(module_accessor: *mut BattleObjectModuleAccessor) -> u32 {
    let entry_id = WorkModule::get_int(module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    1u32 << (entry_id & 7)
}

// Flips `toggled` once at the start of a taunt (rising edge of `in_taunt`).
fn toggle_on_taunt_start(in_taunt: bool, bit: u32, in_flags: &AtomicU32, toggled: &AtomicU32) {
    if in_taunt {
        if in_flags.fetch_or(bit, Ordering::Relaxed) & bit == 0 {
            toggled.fetch_xor(bit, Ordering::Relaxed);
        }
    } else {
        in_flags.fetch_and(!bit, Ordering::Relaxed);
    }
}

unsafe fn apply_meshes(module_accessor: *mut BattleObjectModuleAccessor) {
    let bit = entry_bit(module_accessor);
    let down = DOWN_TOGGLED.load(Ordering::Relaxed) & bit != 0;
    let side = SIDE_TOGGLED.load(Ordering::Relaxed) & bit != 0;

    // Down taunt
    ModelModule::set_mesh_visibility(module_accessor, smash::phx::Hash40::new("hanx123"), down);
    ModelModule::set_mesh_visibility(module_accessor, smash::phx::Hash40::new("shirt"), !down);

    // Side taunt (mask123 follows the same rule as shirt: visible by default, hidden when toggled)
    ModelModule::set_mesh_visibility(module_accessor, smash::phx::Hash40::new("mask123"), !side);
}

// Runs once when the fighter is created at the start of a match: back to the default state.
unsafe extern "C" fn ike_start(fighter: &mut L2CFighterCommon) {
    let module_accessor = fighter.module_accessor;
    let bit = entry_bit(module_accessor);
    DOWN_TOGGLED.fetch_and(!bit, Ordering::Relaxed);
    SIDE_TOGGLED.fetch_and(!bit, Ordering::Relaxed);
    IN_DOWN_TAUNT.fetch_and(!bit, Ordering::Relaxed);
    IN_SIDE_TAUNT.fetch_and(!bit, Ordering::Relaxed);
    apply_meshes(module_accessor);
}

// Runs every frame. The vanilla taunt scripts are left untouched; we only watch the motion.
unsafe extern "C" fn ike_frame(fighter: &mut L2CFighterCommon) {
    let module_accessor = fighter.module_accessor;
    let bit = entry_bit(module_accessor);
    let motion = MotionModule::motion_kind(module_accessor);

    let in_down = motion == hash40("appeal_lw_l") || motion == hash40("appeal_lw_r");
    let in_side = motion == hash40("appeal_s_l") || motion == hash40("appeal_s_r");

    toggle_on_taunt_start(in_down, bit, &IN_DOWN_TAUNT, &DOWN_TOGGLED);
    toggle_on_taunt_start(in_side, bit, &IN_SIDE_TAUNT, &SIDE_TOGGLED);

    // Keep the meshes in the right state every frame (this is what hides `hanx123` by default).
    apply_meshes(module_accessor);
}

pub fn install() {
    Agent::new("ike")
        .on_start(ike_start)
        .on_line(Main, ike_frame)
        .install();
}
