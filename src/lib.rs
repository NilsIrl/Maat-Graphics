//! # maat_engine
//!
//! `maat_engine` is a 2D and 3D graphics engine capabile of running in both
//! `OpenGL` or `Vulkan` graphics api's.
//!
//! Returns a new Flaker based on the specified identifier
//! 
//! # Why Maat?
//! 
//! Maat is derived from the egyption mythology an Egyption word written as m3ˁt.
//! 
//! Established at the creation of the word, Maat distinguishes the world from 
//! the chaos that preceded and surrounds it. Maat encompasses concepts of truth, 
//! balance, order, harmony, law, morality, and justice. Maat was also the 
//! goddess who personified these concepts, and regulated the stars, seasons, 
//! and the actions of mortals and the deities who had brought order from chaos 
//! at the moment of creation.
//!
//! The significance of Maat developed to the point that it embraced all aspects 
//! of existence, including the basic equilibrium of the universe, the 
//! relationship between constituent parts, the cycle of the seasons, heavenly 
//! movements, religious observations and fair dealings, honesty and truthfulness 
//! in social interactions.
//! 
//! Maat bound all things together in an indestructible unity: the universe, 
//! the natural world, the state, and the individual were all seen as parts of 
//! the wider order generated by Maat.
//! 
//! `Order from Chaos`
//! 
//! # Core
//!
//! * `CoreRender` - Is the bridge between openGL and Vulkan provides users with
//!  all needed commands
//!
//! # Examples
//! 
//! extern crate maat_engine as maat;
//!
//! use maat::graphics::CoreRender;
//! use maat::rawvk::RawVk;
//! use maat::rawgl::RawGl;
//! 
//! let use_vulkan = true;
//! 
//! let mut graphics: Box<CoreRender>;
//! 
//! if use_vulkan {
//!   graphics = Box::new(RawVk::new().with_title(String::from("Example Vk")));
//! } else {
//!   graphics = Box::new(RawGl::new().with_title(String::from("Example Gl")));
//! }
//! 
//! // Call Load textures and models here
//! //....
//! 
//! graphics.load_shaders();
//! graphics.init();
//! 
//! // Call dynamic load function here, unless all load calls were preload
//! 
//! let mut draw_calls: Vec<DrawCall> = Vec::with_capacity(100);
//! 
//! loop {
//!   graphics.clear_screen();
//!   
//!   // Update your program here and fill draw_calls with DrawCall commands
//!   
//!   graphics.pre_draw();
//!   graphics.draw(&draw_calls);
//!   graphics.post_draw();
//!   
//!   draw_calls.clear();
//!   
//!   let mut resized = false;
//!   graphics.get_events().poll_events( |ev| {
//!     match ev {
//!       winit::WindowEvent{event, .. } => {
//!         match event {
//!           winit::WindowEvent::Resized(_,_) => resized = true,
//!           // Other window events. Such as input
//!           _=> (), 
//!         }
//!       },
//!       _ => (),
//!     }
//!   });
//!   
//!   if resized {
//!     graphics.screen_resized();
//!   }
//! 
//!   graphics.swap_buffers();
//! }
//! 
//! 
//! 
//! 
//! 
//! 

const ENGINE_VERSION: u32 = (0 as u32) << 22 | (6 as u32) << 12 | (0 as u32);

use self::threadpool::ThreadPool;
use self::texture_shader::TextureShader;

pub use crate::core::CoreMaat;
pub use crate::drawcalls::DrawCall;

pub mod graphics;
pub mod settings;
pub mod camera;
pub mod vulkan;
pub mod math;
mod drawcalls;
mod core;
mod texture_shader;
//mod gltf_interpreter;
mod font;
mod threadpool;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
