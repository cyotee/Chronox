use ggez::*;
use ggez::graphics::*;
use simulation::*;
use simulation::petgraph::prelude::*;
use library::*;
use std::f32::consts::PI;

#[derive(Serialize, Deserialize, Debug)]
pub struct RenderConfig{
    pub colors: [u32;MAX_SIDES]
}

struct GlobalResources{
    font: Font,
    num_font: NumericFont,
    small_num_font: NumericFont
}
impl GlobalResources{
    fn new(ctx: &mut Context) -> GameResult<GlobalResources>{
        let font =  graphics::Font::new(ctx, "/Tuffy.ttf", 24)?;
        let num_font = NumericFont::new(ctx, &font)?;
        let small_font =  graphics::Font::new(ctx, "/Tuffy.ttf", 16)?;
        let small_num_font = NumericFont::new(ctx, &small_font)?;
        let g = GlobalResources { font, num_font, small_num_font};
        Ok(g)
    }
}

fn set_col(ctx: &mut Context, conf: &RenderConfig, player: Player) -> GameResult<()>{
    let col = conf.colors[player];
    set_color(ctx, Color::from_rgb_u32(col))?;
    Ok(())
}
pub struct Renderer{
    resources: GlobalResources
}
impl Renderer {
    pub fn new(ctx: &mut Context) -> GameResult<Renderer>{
        let resources = GlobalResources::new(ctx)?;
        Ok(Renderer{resources})
    }
    pub fn render(&self, ctx: &mut Context, conf: &RenderConfig, sim: &Simulation) -> GameResult<()> {
        //Draw edges and army groups moving on them
        for edge_ref in sim.world.edge_references() {
            let s = &sim.world[edge_ref.source()];
            let t = &sim.world[edge_ref.target()];
            let s_loc = gpt(s.loc);
            let t_loc = gpt(t.loc);
            graphics::line(ctx, &[s_loc, t_loc], 2.)?;
            let edge = edge_ref.weight();
            for group in &edge.transfers {
                let f_progress = match group.direction {
                    DIR::FORWARD => (group.progress as f32) / (edge.length as f32),
                    DIR::BACKWARD => 1.0 - ((group.progress as f32) / (edge.length as f32))
                };
                let loc = s_loc + (t_loc - s_loc) * f_progress;
                graphics::circle(ctx, DrawMode::Fill, loc, 16., 0.5)?;
                set_col(ctx, conf, group.player)?;
                self.resources.small_num_font.draw_centered(ctx, loc, group.count)?;
                set_color(ctx, Color::from_rgba(255, 255, 255, 255))?;
            }
        }

        //Draw planets and any armies on them
        for node_ind in sim.world.node_indices() {
            let node = &sim.world[node_ind];
            let node_loc = gpt(node.loc);
            graphics::circle(ctx, DrawMode::Fill, node_loc, node.max_strength as f32, 0.5)?;
            set_col(ctx, conf, node.owner)?;
            graphics::circle(ctx, DrawMode::Line(1.0), node_loc, node.owner_strength as f32, 0.5)?;
            //TODO: handle multi-player count
            let involved = find_sides_node(node);
            if involved.len() == 1 {
                let player = involved[0];
                set_col(ctx, conf, player)?;
                self.resources.num_font.draw_centered(ctx, node_loc, node.count[player])?;
            } else if involved.len() > 1 {
                let count = involved.len() as f32;
                let angle_increment = 2.0*PI/count;
                let mut angle = PI/2.0;
                for player in involved{
                    set_col(ctx, conf, player)?;
                    let loc = node_loc+lendir(16.0, angle);
                    self.resources.num_font.draw_centered(ctx, loc, node.count[player]) ?;
                    angle += angle_increment;
                }
            }
            set_color(ctx, Color::from_rgba(255, 255, 255, 255))?;
        }
        Ok(())
    }
}