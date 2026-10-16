use {
    smash::{
        lua2cpp::*,
        phx::{Hash40, Vector3f},
        app::{lua_bind::*, *},
        lib::lua_const::*,
        hash40,
    },
    smashline::*,
    std::sync::atomic::{AtomicU32, Ordering},
};

// ---------------------------------------------------------------------------------------
// EFFECT / SOUND SETTINGS  (change these to tune or swap things without touching the logic)
// ---------------------------------------------------------------------------------------
// Effect names come from Ike's own ef_ike.eff (lowercase of the names stored in the file).
// Fire effects in that file you can try instead:
//   ike_volcano_hold      flames + sparks around the sword (Eruption charge)   <- used below
//   ike_final_sword_fire  fire on the sword (Final Smash)
//   ike_volcano_max       big fire + smoke column (Eruption release, not subtle)
//   ike_volcano_flash     burst with lines and debris (not subtle)
// The effect starts when the meshes toggle and stops when the taunt ends.
// Set a name to "" to disable that effect / sound.
const DOWN_EFFECT: &str = "ike_volcano_hold";
const DOWN_EFFECT_SIZE: f32 = 1.0;
const DOWN_SOUND: &str = "se_common_guard_on"; // placeholder: test it, swap if silent

const SIDE_EFFECT: &str = "ike_volcano_hold";
const SIDE_EFFECT_SIZE: f32 = 1.0;
const SIDE_SOUND: &str = "se_common_guard_on"; // placeholder: test it, swap if silent

// Bone the effect follows. "haver" should be the sword hand/sword. If the fire shows up in
// the wrong place, try "top" (the character's root) and use EFFECT_OFFSET_Y to raise it.
const EFFECT_BONE: &str = "haver";
const EFFECT_OFFSET_Y: f32 = 0.0;

// ---------------------------------------------------------------------------------------
// STATE  (one bit per player slot, entry id 0..7)
//
// DOWN taunt toggle:  bit 0 = default (`hanx123` hidden, `shirt` visible)
//                     bit 1 = toggled (`hanx123` visible, `shirt` hidden)
// SIDE taunt toggle:  bit 0 = default (`mask123` visible)
//                     bit 1 = toggled (`mask123` hidden)   <- same setting as `shirt`
// ---------------------------------------------------------------------------------------
static DOWN_TOGGLED: AtomicU32 = AtomicU32::new(0);
static SIDE_TOGGLED: AtomicU32 = AtomicU32::new(0);

// Bit = 1 while that player is already inside the taunt (so we toggle only once per taunt).
static IN_DOWN_TAUNT: AtomicU32 = AtomicU32::new(0);
static IN_SIDE_TAUNT: AtomicU32 = AtomicU32::new(0);

#[derive(PartialEq)]
enum TauntEdge {
    Started,
    Ended,
    None,
}

unsafe fn entry_bit(module_accessor: *mut BattleObjectModuleAccessor) -> u32 {
    let entry_id = WorkModule::get_int(module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    1u32 << (entry_id & 7)
}

// Detects the first frame of a taunt (flipping `toggled` once) and the frame it ends.
fn taunt_edge(in_taunt: bool, bit: u32, in_flags: &AtomicU32, toggled: &AtomicU32) -> TauntEdge {
    if in_taunt {
        if in_flags.fetch_or(bit, Ordering::Relaxed) & bit == 0 {
            toggled.fetch_xor(bit, Ordering::Relaxed);
            return TauntEdge::Started;
        }
    } else if in_flags.fetch_and(!bit, Ordering::Relaxed) & bit != 0 {
        return TauntEdge::Ended;
    }
    TauntEdge::None
}

// Starts the toggle effect (following the bone) and plays the toggle sound.
unsafe fn start_toggle_fx(module_accessor: *mut BattleObjectModuleAccessor, effect: &str, size: f32, sound: &str) {
    if !effect.is_empty() {
        EffectModule::req_follow(
            module_accessor,
            Hash40::new(effect),
            Hash40::new(EFFECT_BONE),
            &Vector3f { x: 0.0, y: EFFECT_OFFSET_Y, z: 0.0 },
            &Vector3f { x: 0.0, y: 0.0, z: 0.0 },
            size,
            false,
            0,
            0,
            0,
            0,
            0,
            false,
            false,
        );
    }
    if !sound.is_empty() {
        SoundModule::play_se(module_accessor, Hash40::new(sound), true, false, false, false, enSEType(0));
    }
}

// Stops the effect when the taunt ends (or is interrupted).
unsafe fn stop_toggle_fx(module_accessor: *mut BattleObjectModuleAccessor, effect: &str) {
    if !effect.is_empty() {
        EffectModule::kill_kind(module_accessor, Hash40::new(effect), false, true);
    }
}

unsafe fn apply_meshes(module_accessor: *mut BattleObjectModuleAccessor) {
    let bit = entry_bit(module_accessor);
    let down = DOWN_TOGGLED.load(Ordering::Relaxed) & bit != 0;
    let side = SIDE_TOGGLED.load(Ordering::Relaxed) & bit != 0;

    // Down taunt
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("hanx123"), down);
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("shirt"), !down);

    // Side taunt (mask123 follows the same rule as shirt: visible by default, hidden when toggled)
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("mask123"), !side);
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

    match taunt_edge(in_down, bit, &IN_DOWN_TAUNT, &DOWN_TOGGLED) {
        TauntEdge::Started => start_toggle_fx(module_accessor, DOWN_EFFECT, DOWN_EFFECT_SIZE, DOWN_SOUND),
        TauntEdge::Ended => stop_toggle_fx(module_accessor, DOWN_EFFECT),
        TauntEdge::None => {}
    }
    match taunt_edge(in_side, bit, &IN_SIDE_TAUNT, &SIDE_TOGGLED) {
        TauntEdge::Started => start_toggle_fx(module_accessor, SIDE_EFFECT, SIDE_EFFECT_SIZE, SIDE_SOUND),
        TauntEdge::Ended => stop_toggle_fx(module_accessor, SIDE_EFFECT),
        TauntEdge::None => {}
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
