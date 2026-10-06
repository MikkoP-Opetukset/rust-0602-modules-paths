#![allow(dead_code, unused_variables, unused_mut)]

//! Library crate of the `arena` turn-based combat game.
//!
//! The flat file of the previous demo has been organized into modules. The
//! module tree below starts at the crate root, which is implicitly named
//! `crate`:
//!
//! ```text
//! crate
//! ├── combat
//! │   ├── damage
//! │   └── status
//! ├── items
//! ├── enemy_ai
//! └── ui
//! ```
//!
//! Everything inside a module is private to that module by default. Items in
//! a parent module cannot see the private items of their children, but
//! children can see the private items of their ancestors. Siblings only see
//! each other's `pub` items.
//!
//! Every path in this file is written out in full, which gets long. The next
//! demo shows how `use` shortens them.
//!
//! The modules are `pub` here only so the binary crates, which are separate
//! crates, can reach them. They are declared inline for now; the last demo
//! moves each one to its own file.

/// Rules of the fight: who the fighters are, what they can do, and what
/// happens when they do it.
pub mod combat {
    /// What a fighter chooses to do on its turn.
    ///
    /// Making an enum `pub` makes all of its variants public too. Items can be
    /// handled with an absolute path from the crate root: `crate::items::Item`.
    pub enum Action {
        Attack,
        Defend,
        UseItem(crate::items::Item),
    }

    /// A character in the fight.
    ///
    /// Making a struct `pub` does not make its fields public. Each field is
    /// opted in separately. `hp` and the other private fields can only be
    /// changed by code inside `combat` and its child modules, which keeps
    /// them consistent (for example, HP can never exceed `max_hp`).
    pub struct Fighter {
        pub name: String,
        pub potions: u32,
        pub flasks: u32,
        hp: i32,
        max_hp: i32,
        attack: i32,
        defending: bool,
        status: Option<self::status::StatusEffect>,
    }

    impl Fighter {
        /// Because a private field makes it impossible to build `Fighter`
        /// with a struct literal outside of `combat`, we have to provide a
        /// public constructor, like `Breakfast::summer` in the Rust Book.
        pub fn new(name: &str, max_hp: i32, attack: i32, potions: u32, flasks: u32) -> Fighter {
            Fighter {
                name: String::from(name),
                potions,
                flasks,
                hp: max_hp,
                max_hp,
                attack,
                defending: false,
                status: None,
            }
        }

        // Public getters let other modules read the private fields.
        pub fn hp(&self) -> i32 {
            self.hp
        }

        pub fn max_hp(&self) -> i32 {
            self.max_hp
        }

        pub fn is_alive(&self) -> bool {
            self.hp > 0
        }

        pub fn is_poisoned(&self) -> bool {
            self.status.is_some()
        }

        pub fn heal(&mut self, amount: i32) {
            self.hp = (self.hp + amount).min(self.max_hp);
        }

        pub fn set_status(&mut self, effect: self::status::StatusEffect) {
            self.status = Some(effect);
        }
    }

    /// Applies the action to the fighters and returns a line describing it.
    pub fn resolve_action(actor: &mut Fighter, target: &mut Fighter, action: Action) -> String {
        // We are inside the module that defines `Fighter`, so the private
        // fields are accessible here.
        actor.defending = false;

        // `damage` is a child module; a relative path reaches its `pub` items.
        // Calling its private function from here does not compile:
        //
        // let v = damage::variance();

        match action {
            Action::Attack => {
                let amount = damage::calculate(actor, target);
                target.hp -= amount;
                format!("{} hits {} for {amount} damage.", actor.name, target.name)
            }
            Action::Defend => {
                actor.defending = true;
                format!("{} braces for impact.", actor.name)
            }
            // Absolute path to a function in a sibling module.
            Action::UseItem(item) => crate::items::use_item(item, actor, target),
        }
    }

    /// How hard attacks hit.
    pub mod damage {
        /// Damage dealt by an attack.
        ///
        /// `damage` is a child of `combat`, where `Fighter` is defined, so it
        /// may read the private `attack` and `defending` fields. The `ui`
        /// module may not.
        pub fn calculate(attacker: &super::Fighter, defender: &super::Fighter) -> i32 {
            let mut amount = attacker.attack + variance();
            if defender.defending {
                amount /= 2;
            }
            amount.max(1)
        }

        // Private: an implementation detail that only `damage` can call.
        fn variance() -> i32 {
            rand::random_range(-2..=2)
        }
    }

    /// Effects that last for several turns.
    pub mod status {
        pub enum StatusEffect {
            Poisoned { turns_left: u32 },
        }

        /// Applies the damage over time of the fighter's status effect.
        pub fn tick(fighter: &mut super::Fighter) -> Option<String> {
            if let Some(StatusEffect::Poisoned { turns_left }) = fighter.status {
                fighter.hp -= 3;
                fighter.status = if turns_left > 1 {
                    Some(StatusEffect::Poisoned {
                        turns_left: turns_left - 1,
                    })
                } else {
                    None
                };
                Some(format!("{} suffers 3 poison damage.", fighter.name))
            } else {
                None
            }
        }
    }
}

/// Things fighters can spend during a fight.
pub mod items {
    pub enum Item {
        Potion,
        PoisonFlask,
    }

    pub fn use_item(
        item: Item,
        user: &mut crate::combat::Fighter,
        target: &mut crate::combat::Fighter,
    ) -> String {
        // `items` is a sibling of `combat`, not a child, so `user.hp -= 1`
        // would not compile here. Uncomment it to see the privacy error.
        //
        // user.hp -= 1;

        match item {
            crate::items::Item::Potion if user.potions > 0 => {
                user.potions -= 1;
                user.heal(15);
                format!("{} drinks a potion and recovers 15 HP.", user.name)
            }
            crate::items::Item::PoisonFlask if user.flasks > 0 => {
                user.flasks -= 1;
                target.set_status(crate::combat::status::StatusEffect::Poisoned { turns_left: 3 });
                format!("{} throws a flask. {} is poisoned!", user.name, target.name)
            }
            _ => format!("{} fumbles around: nothing left to use.", user.name),
        }
    }
}

/// The enemy's behavior.
pub mod enemy_ai {
    /// A weighted random choice, with a bias to heal when low on HP.
    pub fn choose_action(enemy: &crate::combat::Fighter) -> crate::combat::Action {
        let roll = rand::random_range(0..100);
        if is_hurt(enemy) && enemy.potions > 0 && roll < 60 {
            crate::combat::Action::UseItem(crate::items::Item::Potion)
        } else if roll < 20 {
            crate::combat::Action::Defend
        } else if enemy.flasks > 0 && roll < 35 {
            crate::combat::Action::UseItem(crate::items::Item::PoisonFlask)
        } else {
            crate::combat::Action::Attack
        }
    }

    // A private helper. Only this module can call it.
    fn is_hurt(fighter: &crate::combat::Fighter) -> bool {
        fighter.hp() < fighter.max_hp() / 3
    }
}

/// The text interface for the player.
pub mod ui {
    pub fn clear_screen() {
        // ANSI escape codes: clear the terminal and move the cursor to the top left.
        print!("\x1B[2J\x1B[1;1H");
    }

    pub fn show_status(player: &crate::combat::Fighter, enemy: &crate::combat::Fighter) {
        for fighter in [player, enemy] {
            // `ui` is not allowed to read `fighter.hp` directly; it has to use
            // the public getters.
            let hp = fighter.hp().max(0);
            let filled = (hp * 20 / fighter.max_hp()) as usize;
            let poisoned = if fighter.is_poisoned() {
                " (poisoned)"
            } else {
                ""
            };
            println!(
                "{:<8} [{}{}] {}/{}{}",
                fighter.name,
                "#".repeat(filled),
                "-".repeat(20 - filled),
                hp,
                fighter.max_hp(),
                poisoned
            );
        }
    }

    /// Asks the player for an action. Returns `None` when the player quits.
    pub fn prompt_action(player: &crate::combat::Fighter) -> Option<crate::combat::Action> {
        loop {
            println!(
                "\n1) Attack  2) Defend  3) Potion ({})  4) Poison flask ({})  q) Quit",
                player.potions, player.flasks
            );
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).unwrap() == 0 {
                return None;
            }
            match input.trim() {
                "1" => return Some(crate::combat::Action::Attack),
                "2" => return Some(crate::combat::Action::Defend),
                "3" => return Some(crate::combat::Action::UseItem(crate::items::Item::Potion)),
                "4" => {
                    return Some(crate::combat::Action::UseItem(
                        crate::items::Item::PoisonFlask,
                    ));
                }
                "q" => return None,
                _ => println!("Unknown choice."),
            }
        }
    }
}
