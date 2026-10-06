//! A second binary crate in the same package: the computer plays both sides.
//! Run it with `cargo run --bin simulate`.

fn main() {
    let mut knight = arena::combat::Fighter::new("Knight", 40, 8, 2, 1);
    let mut goblin = arena::combat::Fighter::new("Goblin", 30, 6, 1, 1);

    for round in 1..=50 {
        println!("--- Round {round} ---");
        if take_turn(&mut knight, &mut goblin) || take_turn(&mut goblin, &mut knight) {
            return;
        }
    }
}

/// Plays one turn and returns `true` when the fight is over.
fn take_turn(actor: &mut arena::combat::Fighter, target: &mut arena::combat::Fighter) -> bool {
    if let Some(message) = arena::combat::status::tick(actor) {
        println!("{message}");
    }
    if !actor.is_alive() {
        println!("{} is defeated!", actor.name);
        return true;
    }
    let action = arena::enemy_ai::choose_action(actor);
    println!("{}", arena::combat::resolve_action(actor, target, action));
    if !target.is_alive() {
        println!("{} is defeated!", target.name);
        return true;
    }
    false
}
