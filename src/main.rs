use macroquad::prelude::*;

fn pythag(a: (f32,f32)) -> f32 {
    return (
        a.0 * a.0 +
        a.1 * a.1
    ).sqrt();
}

fn angle_between(a: (f32,f32), b: (f32,f32)) -> f32 {
    let (x,y) = (b.0-a.0, b.1-a.1);
    let (ax,ay) = (x.abs(),y.abs());
    const pi: f32 = std::f32::consts::PI;
    if x > 0.0 && y >= 0.0 {
        return (ay/ax).atan();
    } else if x <= 0.0 && y > 0.0 {
        return (ax/ay).atan() + pi / 2.0;
    } else if x < 0.0 && y <= 0.0 {
        return (ay/ax).atan() + pi;
    } else {
        return (ax/ay).atan() + pi * 3.0 / 2.0;
    }
}

fn simple_track(speed: f32, time: f32) -> (f32, f32) {
    return (speed * time, 100.0);
}

fn circle_track(speed: f32, time: f32) -> (f32, f32) {
    let radius: f32 = 100.0;
    let ox: f32 = 150.0;
    let oy: f32 = 150.0;
    return (ox + radius * (speed * time/radius).cos(), oy + radius * (speed * time / radius).sin());
}

fn target_first(pos: (f32,f32), enemies: Vec<&Enemy>) -> Option<&Enemy>{
    return enemies.into_iter().nth(0);
}

fn place_any(pos: (f32,f32), radius: f32, towers: Vec<Tower>) -> bool {
    return true;
}

struct Projectilepath {
    cos_angle: f32,
    sin_angle: f32,
    source: (f32,f32),
    update_foo: fn(f32,f32,f32,f32,(f32,f32)) -> (f32,f32),
}

impl Projectilepath {
    fn projectile_straight (source: (f32,f32), target: (f32,f32),) -> Projectilepath {
        let angle: f32 = angle_between(source, target);
        let cos_angle: f32 = angle.cos();
        let sin_angle: f32 = angle.sin();
        fn foo(cos_angle: f32, sin_angle: f32, speed: f32, time: f32, source: (f32,f32)) -> (f32,f32) {
            let distance:f32 = speed * time;
            let dx: f32 = source.0 + distance * cos_angle;
            let dy: f32 = source.1 + distance * sin_angle;
            return (dx,dy);
        }
        return Projectilepath {
            cos_angle: cos_angle,
            sin_angle: sin_angle,
            source: source,
            update_foo: foo,
        }
    }
    fn update(&self, speed: f32, time: f32) -> (f32,f32) {
        let foo = self.update_foo;
        let (x,y) = foo(self.cos_angle,self.sin_angle,speed,time,self.source);
        return (x,y);
    }
}

struct Tri {
    pos1: Vec2,
    pos2: Vec2,
    pos3: Vec2,
    colour: Color,
}

impl Tri {
    fn new(x: f32, y: f32, colour: Color) -> Tri {
        return Tri {
            pos1: Vec2::new(x-10.0,y-10.0),
            pos2: Vec2::new(x+10.0,y-10.0),
            pos3: Vec2::new(x,y+10.0),
            colour: colour,
        }
    }
    fn get_centre(&self) -> (f32,f32) {
        return (
            (self.pos1.x + self.pos2.x + self.pos3.x) / 3.0,
            (self.pos1.y + self.pos2.y + self.pos3.y) / 3.0
        )
    }
    fn move_to(&mut self, x: f32, y: f32) {
        self.pos1 = Vec2::new(x-10.0,y-10.0);
        self.pos2 = Vec2::new(x+10.0,y-10.0);
        self.pos3 = Vec2::new(x,y+10.0);
    }
    fn move_by(&mut self, x: f32, y: f32) {
        let (ox,oy) = self.get_centre();
        self.move_to(x + ox, y + oy);
    }
    fn draw(&self) {
        draw_triangle(self.pos1,self.pos2,self.pos3,self.colour);
    }
}
struct Player {
    health: u32,
    money: u32,
    path: fn(f32,f32) -> (f32,f32),
    def_target: fn((f32,f32),Vec<&Enemy>) -> Option<&Enemy>,
    enemies: Vec<Enemy>,
    projectiles: Vec<Projectile>,
    towers: Vec<Tower>,
    difficulty: u32,
    mouse_state: bool,
}

impl Player {
    fn new(difficulty: u32, path: fn(f32, f32) -> (f32,f32),def_target: fn((f32, f32),Vec<&Enemy>) -> Option<&Enemy>) -> Player {
        let mut n_health: u32 = 200 - difficulty * 50;
        if n_health < 1 {
            n_health = 1;
        }
        let n_money: u32  = 1000 - difficulty * 100;
        let enemies: Vec<Enemy> = vec![];
        let projectiles: Vec<Projectile> = vec![];
        let towers: Vec<Tower> = vec![];
        return Player {
            health: n_health,
            money: n_money,
            path: path,
            def_target: target_first,
            enemies: enemies,
            projectiles: projectiles,
            towers: towers,
            difficulty: difficulty,
            mouse_state: false,
        }
    }
    fn new_enemy(&mut self, health: u32) {
        let n_enemy = Enemy::new(health);
        self.enemies.push(n_enemy);
    }
    fn new_tower(&mut self, x: f32, y: f32, target: fn((f32,f32),Vec<&Enemy>) -> Option<&Enemy>, placement: fn((f32,f32),f32,Vec<Tower>) -> bool, radius: f32) {
        let n_tower = Tower::new(x,y,target,placement,radius);
        self.towers.push(n_tower);
    }
    fn shoot_enemy(&mut self, enemy: Enemy,source: (f32,f32)) {
        let target = (enemy.x,enemy.y);
        self.new_projectile(source,target,5.0,1,1,10.0);
    }
    fn new_projectile(&mut self, source: (f32,f32), target: (f32,f32), speed: f32, pierce: u32, damage: u32, radius: f32) {
        let n_projectile = Projectile::new(source,target,speed,pierce,damage,radius);
        self.projectiles.push(n_projectile);
    }

    // Updates enemies, towers and projectiles
    fn update(&mut self, dt: f32) {

        let mut enemy_ref: Vec<&Enemy> = vec![];

        // Update enemies
        for enemy in &mut self.enemies {
            enemy.update(dt,self.path);
            
        }

        // Get enemies for processing projectile creation
        for enemy in &self.enemies {
            enemy_ref.push(enemy);
        }

        // Holds the player position and a foo to get the target position
        let mut projectile_to_make: Vec<((f32,f32),fn ((f32,f32), Vec<&Enemy>) -> Option<&Enemy>)> = vec![];

        // Holds the tower positions and enemy positions 
        let mut projectile_target: Vec<((f32,f32),(f32,f32))> = vec![];
        // Update towers
        for tower in &mut self.towers {
            if tower.can_shoot(dt) {
                let tower_pos = tower.get_pos();
                let target_function = tower.get_target();
                projectile_to_make.push((tower_pos,target_function));
            }
        }

        // Get projectile targets
        for point in projectile_to_make {
            let tower_pos = point.0;
            let foo = point.1;
            let enemy = foo(tower_pos, enemy_ref.clone());
            match enemy {
                Some(target_enemy) => projectile_target.push((tower_pos,(target_enemy.x,target_enemy.y))),
                None => {},
            }
        }

        for point in projectile_target {
            let source = point.0;
            let target = point.1;
            self.new_projectile(source,target,50.0,5,5,5.0)
        }
        
        for projectile in &mut self.projectiles {
            projectile.update(dt);
        }
    }
    fn on_tick(&mut self) {
        self.new_enemy(100);
    }
    fn input(&mut self) {
        if is_mouse_button_down(MouseButton::Left) {
            if self.mouse_state == false {
                let (mx,my) = mouse_position();
                self.new_tower(mx,my,target_first,place_any,15.0);
            }
            self.mouse_state = true;
        }
        else {
            self.mouse_state = false;
        }
    }
    fn draw(&self) {
        for i in self.enemies.iter() {
            i.draw();
        }
        for i in self.towers.iter() {
            i.draw();
        }
        for i in self.projectiles.iter() {
            i.draw();
        }
    }
}


struct Enemy {
    health: u32,
    reward: u32,
    speed: f32,
    time: f32,
    x: f32,
    y: f32,
    radius: f32,
    tri: Tri,
}

impl Enemy {
    fn new(health: u32) -> Enemy {
        let reward: u32 = 1;
        let speed: f32 = (health as f32) * 1.5;
        let time: f32 = 0.0;
        let x: f32 = 0.0;
        let y: f32 = 0.0;
        let radius: f32 = 10.0 + (health as f32) * 2.0;
        let tri: Tri = Tri::new(x,y,RED);
        return Enemy {
            health: health,
            reward: reward,
            speed: speed,
            time: time,
            x: x,
            y: y,
            radius: radius,
            tri: tri,
        };
    }
    fn draw(&self) {
        self.tri.draw()
    }
    fn path(&mut self, path: fn(f32, f32) -> (f32,f32)) {
        let npos: (f32,f32) = path(self.speed, self.time);
        
        // let offset: Vec2 = tup_to_vec2(npos) - Vec2::new(self.x,self.y);

        self.tri.move_to(npos.0, npos.1);
        (self.x,self.y) = npos
    }
    fn update(&mut self, dt: f32, path: fn(f32, f32) -> (f32,f32)) {
        self.time += dt;
        self.path(path);
    }
}
struct Projectile {
    x: f32,
    y: f32,
    source: (f32,f32),
    target: (f32,f32),
    time: f32,
    speed: f32,
    path: Projectilepath,
    pierce: u32,
    damage: u32,
    tri: Tri,
    radius: f32,
}
impl Projectile {
    fn new(source: (f32,f32), target: (f32,f32), speed: f32, pierce: u32, damage: u32, radius: f32) -> Projectile {
        
        let projectile_path: Projectilepath = Projectilepath::projectile_straight(source, target);
        let tri = Tri::new(source.0,source.1,YELLOW);
        return Projectile {
            x: source.0,
            y: source.1,
            source: source,
            target: target,
            time: 0.0,
            speed: speed,
            path: projectile_path,
            pierce: pierce,
            damage: damage,
            tri: tri,
            radius: radius,
        }
    }
    fn update(&mut self,dt: f32) {
        self.time += dt;
        let (nx,ny) = self.path.update(self.speed, self.time);
        self.tri.move_to(nx,ny);
        (self.x,self.y) = (nx,ny);

    }
    fn draw(&self) {
        self.tri.draw();
    }
}
struct Tower {
    x: f32,
    y: f32,
    target: fn((f32,f32), Vec<&Enemy>) -> Option<&Enemy>,
    placement: fn((f32,f32),f32,Vec<Tower>) -> bool,
    tri: Tri,
    radius: f32,
    max_cooldown: f32,
    cooldown: f32,
}

impl Tower {
    fn new(x: f32, y: f32, target: fn((f32,f32),Vec<&Enemy>) -> Option<&Enemy>, placement: fn((f32,f32),f32,Vec<Tower>) -> bool, radius: f32) -> Tower {
        let tri = Tri::new(x,y,BLUE);
        return Tower {
            x: x,
            y: y,
            target: target,
            placement: placement,
            tri: tri,
            radius: radius,
            max_cooldown: 0.5,
            cooldown: 0.5,
        }
    }

    // Returns true when the cooldown period elapses
    fn can_shoot(&mut self, dt: f32) -> bool {
        self.cooldown -= dt;
        if self.cooldown < 0.0 {
            self.cooldown = self.max_cooldown;
            return true;
        }
        return false;
    }
    // Returns the position of the tower
    fn get_pos(&self) -> (f32,f32) {
       return (self.x,self.y);
    }

    // Returns the targetting function
    fn get_target(&self) -> fn ((f32, f32),Vec<&Enemy>) -> Option<&Enemy> {
        return self.target;
    }

    // Draws the tower
    fn draw(&self) {
        self.tri.draw()
    }
}

#[macroquad::main("Blons TD")]
async fn main() {
    let mut player: Player = Player::new(1,circle_track,target_first);
    let mut dt: f32;
    let mut game_time: f64;
    let mut tick: f32 = 0.0;
    let mut tick_time: f32 = 1.0;
    loop {
        dt = get_frame_time();
        tick += dt;
        game_time = get_time();

        if tick > tick_time {
            tick -= tick_time;
            tick_time *= 0.9;
            player.on_tick();
        }
        

        clear_background(BLACK);

        player.update(dt);

        player.input();
 
        player.draw();

        next_frame().await
    }
}