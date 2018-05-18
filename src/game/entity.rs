
extern crate ggez;
extern crate rand;

use ggez::graphics;
use ggez::Context;
use game::MainState;
use game::ENEMY_BULLET_COOLDOWN;
use game::DISABLE_SFX;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum EntityType {
	Boss,
	EnemyBullet,
	PlayerBullet,
	Enemy,
	Player,
	Powerup,
}

#[derive(Debug)]
pub enum Lifetime {
	Forever,
	Milliseconds(i64),
}

// An entity has one of three movement types:
// - None: The entity is static on screen (text/effects)
// - Linear: The entity has a constant x and y velocity.
// - Generated: The entity will use the lambda function to generate an x
// and y translation value every time it updates. The first parameter is
// the ms elapsed since the entity spawned, the second is a random number
// generator, and the third is a unique seed value between -1.0 and 1.0.
pub enum Movement {
	None,
	Linear(f32, f32),
	Generated(fn(u64,&mut rand::ThreadRng, f64)->(f32, f32)),
}

pub struct Entity {
	pub text: graphics::Text,
	pub entity_type: EntityType,
    pub x: f32,
    pub y: f32,
    pub hp: u8,
	pub dam: u8,
    pub vel: f32,
	pub movement: Movement,
	pub bounds: graphics::Rect,
	pub lifetime: Lifetime,
	pub seed: f64,
	pub timer: u64,
	pub bullet_cooldown: i64,
	pub angle: f32,
}

impl Entity {
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
    }

	pub fn update(&mut self, delta_ms: u64, state: &mut MainState, _ctx: &mut Context) {
		
		
		// Update lifetimes
		self.timer += delta_ms;
		self.lifetime = match self.lifetime {
			Lifetime::Forever => Lifetime::Forever,
			Lifetime::Milliseconds(remaining) => Lifetime::Milliseconds(remaining - delta_ms as i64),
		};

		// Process bullet cooldowns
		self.bullet_cooldown -= delta_ms as i64;
		if self.bullet_cooldown < 0 {
			self.bullet_cooldown = 0;
		}
	
		// Process movements
		let delta_time = delta_ms as f32 / 1000_f32;
		match self.movement {
			Movement::None => (),
			Movement::Linear(x,y) => self.translate(x * delta_time, y * delta_time),
			Movement::Generated(func) => {
				let (x, y) = func(self.timer, &mut state.rng.clone(), self.seed);
				self.translate(x * delta_time, y * delta_time);
				},
		}
	

		match self.entity_type {

			// Player only code
			// This handles the player movements
			EntityType::Player => {
				
				let vel= self.vel * (delta_ms as f32 / 1000_f32);

				match (state.input.up, state.input.right, state.input.down, state.input.left) {
					( true, false, false, false) => self.translate(0.0, -vel),
					( true,  true, false, false) => self.translate(vel*0.707, -vel*0.707),
					(false,  true, false, false) => self.translate(vel, 0.0),
					(false,  true,  true, false) => self.translate(vel*0.707, vel*0.707),
					(false, false,  true, false) => self.translate(0.0, vel),
					(false, false,  true,  true) => self.translate(-vel*0.707, vel*0.707),
					(false, false, false,  true) => self.translate(-vel, 0.0),
					( true, false, false,  true) => self.translate(-vel*0.707, -vel*0.707),
					_ => (),
				}

				// Limit player position to map.
				let window_width = _ctx.conf.window_mode.width as f32;
				let window_height = _ctx.conf.window_mode.height as f32;

				if self.x + self.bounds.x < 0.0 {
					self.x = 0.0 - self.bounds.x;
				}
				if self.x + self.bounds.x + self.bounds.w > window_width {
					self.x = window_width - (self.bounds.x + self.bounds.w);
				}
				if self.y + self.bounds.y < 0.0 {
					self.y = 0.0 - self.bounds.y;
				}
				if self.y + self.bounds.y + self.bounds.h > window_height {
					self.y = window_height - (self.bounds.y + self.bounds.h);
				}

	
				
			},

			// Enemy only code
			EntityType::Enemy => {
				if self.bullet_cooldown <= 0 {
					self.bullet_cooldown = ENEMY_BULLET_COOLDOWN;
					//enemy_bullet_spawner(self, self.x, self.y);
					let eb = state.spawner.spawn_enemy_bullet(self.x, self.y);
					state.entities.push(eb);

				}
			},

			// Boss only code
			EntityType::Boss => (),

			// Player bullet code
			EntityType::PlayerBullet => {
				self.angle += delta_ms as f32 / 100.0;
			},

			// Enemy bullet code
			EntityType::EnemyBullet => (),

			// Powerup codes
			EntityType::Powerup => (),
		}
		
	}
}