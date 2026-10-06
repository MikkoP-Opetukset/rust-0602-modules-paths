//! Binary crate of the package: the interactive game.
//!
//! From here the library is reached through its crate name, `arena`, and
//! then down the module tree. Only public items are visible: `arena::combat`
//! is `pub`, `arena::combat::damage::calculate` is `pub` too, but
//! `arena::combat::damage::variance` is private and could not be called.

fn main() {
    let mut player = arena::combat::Fighter::new("Hero", 40, 8, 2, 1);
    let mut enemy = arena::combat::Fighter::new("Goblin", 30, 6, 1, 1);
    let mut log = String::new();

    // Struct literals do not work from here because `hp` is private, so this
    // does not compile:
    //
    // let cheater = arena::combat::Fighter {
    //     name: String::from("Cheater"),
    //     hp: 999,
    //     ..
    // };

    loop {
        arena::ui::clear_screen();
        arena::ui::show_status(&player, &enemy);
        println!("\n{log}");
        log.clear();

        if let Some(message) = arena::combat::status::tick(&mut player) {
            log.push_str(&format!("{message}\n"));
        }
        if !player.is_alive() {
            println!("{log}You were defeated.");
            break;
        }

        let Some(action) = arena::ui::prompt_action(&player) else {
            println!("You flee the arena.");
            break;
        };
        log.push_str(&arena::combat::resolve_action(
            &mut player,
            &mut enemy,
            action,
        ));
        log.push('\n');

        if let Some(message) = arena::combat::status::tick(&mut enemy) {
            log.push_str(&format!("{message}\n"));
        }
        if !enemy.is_alive() {
            println!("{log}The {} falls. Victory!", enemy.name);
            break;
        }

        let action = arena::enemy_ai::choose_action(&enemy);
        log.push_str(&arena::combat::resolve_action(
            &mut enemy,
            &mut player,
            action,
        ));
        log.push('\n');
    }
}
