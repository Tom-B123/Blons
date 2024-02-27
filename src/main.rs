use macroquad::prelude::*;

const PI: f32 = std::f32::consts::PI;

fn speed_from_health(health: u32) -> f32 {
    match health {
        1 => return 100.0,
        2 => return 200.0,
        3..=10 => return 300.0,
        11.. => return 400.0,
        _ => return 0.0,
    }
}

fn pythag(a: (f32,f32), b: (f32,f32)) -> f32 {
    let x = a.0 - b.0;
    let y = a.1 - b.1;
    return x * x + y * y;
}

fn pythag_sqrt(a: (f32,f32), b: (f32,f32)) -> f32 {
    let x = a.0 - b.0;
    let y = a.1 - b.1;
    return (x * x + y * y).sqrt();
}

fn angle_between(a: (f32,f32), b: (f32,f32)) -> f32 {
    let (x,y) = (b.0-a.0, b.1-a.1);
    let (ax,ay) = (x.abs(),y.abs());
    
    if x > 0.0 && y >= 0.0 {
        return (ay/ax).atan();
    } else if x <= 0.0 && y > 0.0 {
        return (ax/ay).atan() + PI / 2.0;
    } else if x < 0.0 && y <= 0.0 {
        return (ay/ax).atan() + PI;
    } else {
        return (ax/ay).atan() + PI * 3.0 / 2.0;
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

fn target_first(pos: (f32,f32), enemies: Vec<&Enemy>, range: f32) -> Option<&Enemy>{
    let mut furthest_dist: f32 = 0.0;
    let mut within: Vec<&Enemy> = vec![];
    for enemy in enemies {
        let distance: f32 = pythag(pos, (enemy.x,enemy.y));
        if distance < range * range {
            within.push(enemy);
        }
    }
    let mut target: Option<&Enemy> = None;
    for enemy in within {
        let distance = enemy.time * enemy.speed;
        if distance > furthest_dist {
            furthest_dist = distance;
            target = Some(enemy);
        }
        
    }
    return target;
}

fn place_any(pos: (f32,f32), radius: f32, towers: Vec<Tower>) -> bool {
    return true;
}

struct Projectilepath {
    angle: f32,
    source: (f32,f32),
    target: (f32,f32),
    update_foo: fn(f32,f32,f32,(f32,f32), (f32,f32)) -> (f32,f32),
}

impl Projectilepath {
    fn projectile_straight (source: (f32,f32), target: (f32,f32),) -> Projectilepath {
        let angle: f32 = angle_between(source, target);
        
        fn foo(angle: f32, speed: f32, time: f32, source: (f32,f32), target: (f32,f32)) -> (f32,f32) {
            let cos_angle: f32 = angle.cos();
            let sin_angle: f32 = angle.sin();
            let distance:f32 = speed * time;
            let dx: f32 = source.0 + distance * cos_angle;
            let dy: f32 = source.1 + distance * sin_angle;
            return (dx,dy);
        }
        return Projectilepath {
            angle: angle,
            source: source,
            target: target,
            update_foo: foo,
        }
    }
    fn projectile_circle (source: (f32,f32), target: (f32,f32),) -> Projectilepath {
        let angle: f32 = angle_between(source, target);
        
        fn foo(angle: f32, speed: f32, time: f32, source: (f32,f32), target: (f32,f32)) -> (f32,f32) {
            let cos_angle: f32 = (angle + time * speed * PI / 120.0).cos();
            let sin_angle: f32 = (angle + time * speed * PI / 120.0).sin();
            let dx: f32 = target.0 + 35.0 * cos_angle;
            let dy: f32 = target.1 + 35.0 * sin_angle;
            return (dx,dy);
        }
        return Projectilepath {
            angle: angle,
            source: source,
            target: target,
            update_foo: foo,
        }
    }

    fn update(&self, speed: f32, time: f32) -> (f32,f32) {
        let foo = self.update_foo;
        let (x,y) = foo(self.angle,speed,time,self.source, self.target);
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
    def_target: fn((f32,f32),Vec<&Enemy>, f32) -> Option<&Enemy>,
    enemies: Vec<Enemy>,
    projectiles: Vec<Projectile>,
    towers: Vec<Tower>,
    difficulty: u32,
    mouse_state: bool,
}

impl Player {
    fn new(difficulty: u32, path: fn(f32, f32) -> (f32,f32),def_target: fn((f32, f32),Vec<&Enemy>,f32) -> Option<&Enemy>) -> Player {
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
            def_target: def_target,
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
    fn new_tower(&mut self, x: f32, y: f32, target: fn((f32,f32),Vec<&Enemy>,f32) -> Option<&Enemy>, placement: fn((f32,f32),f32,Vec<Tower>) -> bool, range: f32, radius: f32) {
        let n_tower = Tower::new(x,y,target,placement,range, radius);
        self.towers.push(n_tower);
    }
    fn new_projectile(&mut self, source: (f32,f32), target: (f32,f32), speed: f32, pierce: u32, damage: u32, radius: f32) {
        let n_projectile = Projectile::new(source,target,speed,pierce,damage,radius);
        self.projectiles.push(n_projectile);
    }

    fn remove_projectile(&mut self, pos: usize) {
        if pos >= self.projectiles.len() { 
            // println!("proj pos: {}, should be < {}",pos, self.projectiles.len());
            return;
        }
        self.projectiles.remove(pos);
    }

    fn remove_enemy(&mut self, pos: usize) {
        if pos >= self.enemies.len() { 
            // println!("enemy pos: {}, should be < {}",pos, self.enemies.len());
            return; 
        }
        self.enemies.remove(pos);
    }

    // Returns a vector of enemy indicies and updated health values, then projectile indicies and updated pierce values.
    fn enemies_hit(&mut self) -> (Vec<(usize,u32)>,Vec<(usize,u32)>){
        // The index of the enemy to adjust
        let mut enemy_pos: usize = 0;
        let mut projectile_pos: usize = 0;

        // The position and health of enemies
        let mut enemies: Vec<(f32,f32,u32)> = vec![];

        // The position, pierce and damage of the projectile
        let mut projectiles: Vec<(f32,f32,u32,u32)> = vec![];

        // Stores the enemy index and new health, then the bullet index and new pierce value
        let mut out: (Vec<(usize,u32)>,Vec<(usize,u32)>) = (vec![],vec![]);
        for enemy in &self.enemies {
            enemies.push((enemy.x,enemy.y,enemy.health));
        }
        for projectile in &self.projectiles {
            projectiles.push((projectile.x,projectile.y,projectile.pierce, projectile.damage));
        }
        for enemy in &enemies {
            for projectile in &projectiles {
                if pythag((enemy.0,enemy.1),(projectile.0,projectile.1)) < 100.0 {
                    // Push the enemy index and health - projectile damage
                    out.0.push((enemy_pos,enemy.2 - projectile.3));

                    // Push the projectile position and pierce remaining
                    out.1.push((projectile_pos, projectile.2 - 1));
                }
                projectile_pos += 1;
            }
            projectile_pos = 0;
            enemy_pos += 1;
        }
        return out;
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

        // Holds the tower positions and enemy positions 
        let mut projectile_target: Vec<((f32,f32),(f32,f32))> = vec![];
        
        // Update towers
        for tower in &mut self.towers {
            if tower.can_shoot(dt) {
                let tower_pos = tower.get_pos();
                let target_function = tower.get_target();
                let enemy: Option<&Enemy> = target_function(tower_pos, enemy_ref.clone(), tower.range);
                match enemy {
                    Some(target_enemy) => {
                        projectile_target.push((tower_pos,(target_enemy.x,target_enemy.y)));
                        tower.reset_cooldown();
                    },
                    None => {},
                }
            }
        }

        for point in projectile_target {
            let source = point.0;
            let target = point.1;
            self.new_projectile(source,target,250.0,2,1,5.0)
        }

        let mut projectiles_to_remove: Vec<usize> = vec![];
        let mut pos: usize = 0;

        for projectile in &mut self.projectiles {
            if projectile.update(dt) {
                projectiles_to_remove.push(pos);
            }
            pos += 1;
        }

        for pos in projectiles_to_remove {
            self.remove_projectile(pos);
        }

        let hits: (Vec<(usize,u32)>,Vec<(usize,u32)>) = self.enemies_hit();

        // Update enemy health and remove enemies with 0 health
        for enemy in hits.0 {
            if enemy.0 < self.enemies.len() {
                self.enemies[enemy.0].health = enemy.1;
                if enemy.1 <= 0 {
                    self.remove_enemy(enemy.0);
                }
            }
        }

        // Update projectile pierce and remove projectiles with 0 pierce
        for projectile in hits.1 {
            if projectile.0 < self.projectiles.len() {
                self.projectiles[projectile.0].pierce = projectile.1;
                if projectile.1 <= 0 {
                    self.remove_projectile(projectile.0);
                }
            }
        }
    }
    fn on_tick(&mut self) {
        self.new_enemy(1);
    }
    fn input(&mut self) {
        if is_mouse_button_down(MouseButton::Left) {
            if self.mouse_state == false {
                let (mx,my) = mouse_position();
                let range: f32 = 100.0;
                self.new_tower(mx,my,target_first,place_any,range, 15.0);
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
        let speed: f32 = speed_from_health(health);
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
    lifetime: f32,
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
        let lifetime: f32 = 0.5;
        let projectile_path: Projectilepath = Projectilepath::projectile_straight(source, target);
        let tri = Tri::new(source.0,source.1,YELLOW);
        return Projectile {
            x: source.0,
            y: source.1,
            source: source,
            target: target,
            lifetime: lifetime,
            time: 0.0,
            speed: speed,
            path: projectile_path,
            pierce: pierce,
            damage: damage,
            tri: tri,
            radius: radius,
        }
    }
    fn equals(&self, other: &Projectile) -> bool {
        return self.x == other.x && self.y == other.y && self.target == other.target && self.source == other.source;
    }
    fn update(&mut self,dt: f32) -> bool {
        self.time += dt;
        let (nx,ny) = self.path.update(self.speed, self.time);
        self.tri.move_to(nx,ny);
        (self.x,self.y) = (nx,ny);
        if self.time >= self.lifetime {
            return true;
        }
        return false;
    }
    fn draw(&self) {
        self.tri.draw();
    }
}
struct Tower {
    x: f32,
    y: f32,
    target: fn((f32,f32), Vec<&Enemy>,f32) -> Option<&Enemy>,
    placement: fn((f32,f32),f32,Vec<Tower>) -> bool,
    tri: Tri,
    radius: f32,
    range: f32,
    max_cooldown: f32,
    cooldown: f32,
}

impl Tower {
    fn new(x: f32, y: f32, target: fn((f32,f32),Vec<&Enemy>,f32) -> Option<&Enemy>, placement: fn((f32,f32),f32,Vec<Tower>) -> bool, range: f32, radius: f32) -> Tower {
        let tri = Tri::new(x,y,BLUE);
        return Tower {
            x: x,
            y: y,
            target: target,
            placement: placement,
            tri: tri,
            range: range,
            radius: radius,
            max_cooldown: 0.5,
            cooldown: 0.5,
        }
    }

    // Returns true when the cooldown period elapses
    fn can_shoot(&mut self, dt: f32) -> bool {
        self.cooldown -= dt;
        if self.cooldown < 0.0 {
            return true;
        }
        return false;
    }
    fn reset_cooldown(&mut self) {
        self.cooldown = self.max_cooldown;
    }
    // Returns the position of the tower
    fn get_pos(&self) -> (f32,f32) {
       return (self.x,self.y);
    }

    // Returns the targetting function
    fn get_target(&self) -> fn ((f32, f32),Vec<&Enemy>, f32) -> Option<&Enemy> {
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
    let mut _game_time: f64;
    let mut tick: f32 = 0.0;
    let mut tick_time: f32 = 1.0;
    loop {
        dt = get_frame_time();
        tick += dt;
        _game_time = get_time();

        if tick > tick_time {
            tick -= tick_time;
            tick_time *= 0.95;
            player.on_tick();
        }
        

        clear_background(BLACK);

        player.update(dt);

        player.input();
 
        player.draw();

        next_frame().await
    }
}