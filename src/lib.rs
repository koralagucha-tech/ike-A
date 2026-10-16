#![feature(proc_macro_hygiene)]

mod ike;

#[skyline::main(name = "ike_taunt")]
pub fn main() {
    ike::install();
}
