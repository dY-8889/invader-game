mod enemy;
mod player;

pub use enemy::*;
pub use player::*;

#[macro_export]
macro_rules! method_impl {
    ($player:ty, $attack:ty, $event:ty) => {
        impl PlayerMethod for $player {
            fn hp(&self) -> isize {
                self.0.hp
            }
            fn get_kill(&self) -> usize {
                self.0.kill_count
            }
            fn damage(&mut self, damage: isize) {
                self.0.hp -= damage
            }
            fn kill(&mut self) {
                self.0.kill_count += 1
            }
            fn get_attack(&self) -> AttackType {
                self.0.attack_type
            }
            fn change_attack(&mut self, attack_type: AttackType) {
                self.0.attack_type = attack_type
            }
        }

        impl AttackMethod for $attack {
            #[inline]
            fn new(attack: AttackType) -> Self {
                Self(Attack::new(attack))
            }
            fn attack(&self) -> AttackType {
                self.0.attack
            }
            fn hp(&self) -> isize {
                self.0.hp
            }
            fn damage(&mut self, damage: isize) {
                self.0.hp -= damage
            }
        }

        impl DamageEventMethod for $event {
            fn power(&self) -> isize {
                self.attack.power()
            }
            fn event(attack: AttackType) -> Self {
                Self { attack }
            }
        }
    };
}
