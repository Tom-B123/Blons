use macroquad::prelude::*;

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
    fn draw(&self) {
        draw_triangle(self.pos1,self.pos2,self.pos3,self.colour);
    }
}

#[macroquad::main("Blons TD")]
async fn main() {

    let mut triangle = Tri::new(50.0,50.0,RED);
    loop {
        clear_background(BLACK);

        
        triangle.draw();

        next_frame().await
    }
}