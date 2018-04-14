use gl;

use winit;
use winit::EventsLoop;

use settings::Settings;

use vulkano_win_updated::VkSurfaceBuild;
use vulkano_win_updated::required_extensions;
use vulkano_win_updated as vulkano_win;

use vulkano::device::Queue;
use vulkano::device::Device;
use vulkano::format;
use vulkano::instance::Instance;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::Surface;
use vulkano::image::SwapchainImage;
use vulkano::swapchain::PresentMode;
use vulkano::instance::PhysicalDevice;
use vulkano::device::DeviceExtensions;
use vulkano::swapchain::SurfaceTransform;
use vulkano::swapchain::SwapchainCreationError;

use vulkano::swapchain::CompositeAlpha;

use std::mem;
use std::sync::Arc;

use glutin;
use glutin::GlContext;

pub struct VkWindow {
  events: EventsLoop,
  surface: Arc<Surface<winit::Window>>,
  queue: Arc<Queue>,
  device: Arc<Device>,
  swapchain: Arc<Swapchain<winit::Window>>,
  images: Vec<Arc<SwapchainImage<winit::Window>>>,
}

pub struct GlWindow {
  events: glutin::EventsLoop,
  window: glutin::GlWindow,
}

impl GlWindow {
  pub fn new(width: u32, height: u32, min_width: u32, min_height: u32, fullscreen: bool) -> GlWindow {
    println!("Using openGL");
    
    glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3));
    
    let events_loop = glutin::EventsLoop::new();
    let window = {
      let temp_window: glutin::WindowBuilder;
       
      if fullscreen {
       let monitor = {
         for (num, monitor) in events_loop.get_available_monitors().enumerate() {
           println!("Monitor #{}: {:?}", num, monitor.get_name());
         }

          let monitor = events_loop.get_available_monitors().nth(0).expect("Please enter a valid ID");

          println!("Using {:?}", monitor.get_name());

          monitor
        };
        // Fullscreen
        temp_window = glutin::WindowBuilder::new().with_fullscreen(Some(monitor))
                                           .with_title("OpenGl Fullscreen")
      } else {
        // Windowed
        temp_window = glutin::WindowBuilder::new()
                                            .with_title("OpenGl Windowed").with_decorations(true)
                                            .with_dimensions(width, height)
                                            .with_min_dimensions(min_width, min_height);
      }
      temp_window
    };
    
    let context = glutin::ContextBuilder::new().with_vsync(true).with_multisampling(4);/*{
      let mut temp_context: glutin::ContextBuilder;
        temp_context: glutin::ContextBuilder::new().with_vsync(true).with_multisampling(8)
        
        let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
        println!("Pixel format of the window's GL context: {:?}", gl_window.get_pixel_format());
      temp_context
    };*/
    
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
    println!("Pixel format of the window's GL context: {:?}", gl_window.get_pixel_format());
    unsafe {
      gl_window.make_current().unwrap();
    }
    
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
    
    println!("hidpi: {}", gl_window.hidpi_factor());
    
    GlWindow {
      events: events_loop,
      window: gl_window,
    }
  }
  
  /// Sets the title of the window
  pub fn set_title(&mut self, title: String) {
    self.window.set_title(&title);
  }
  
  /// Returns the dimensions of the window as u32
  pub fn get_dimensions(&self) -> [u32; 2] {
    let (width, height) = self.window.get_inner_size().unwrap();
    [width as u32, height as u32]
  }
  
  /// Returns a reference to the events loop
  pub fn get_events(&mut self) -> &mut winit::EventsLoop {
    &mut self.events
  }
  
  /// Swaps the drawing buffer
  pub fn swap_buffers(&mut self) {
    self.window.swap_buffers().unwrap();
  }
  
  /// Resizes the current window
  pub fn resize_screen(&mut self, dimensions: [u32; 2]) {
    self.window.resize(dimensions[0], dimensions[1]);
  }
  
  /// Returns the current dpi scale factor
  ///
  /// Needed to solve issues with Hidpi monitors
  pub fn get_dpi_scale(&self) -> f32 {
    self.window.hidpi_factor()
  }
  
  /// Enables the cursor to be drawn whilst over the window
  pub fn show_cursor(&mut self) {
    self.window.set_cursor(winit::MouseCursor::Default);
  }
  
  /// Disables the cursor from being drawn whilst over the window
  pub fn hide_cursor(&mut self) {
    self.window.set_cursor(winit::MouseCursor::NoneCursor);
  }
}

impl VkWindow {
  pub fn new(width: u32, height: u32, min_width: u32, min_height: u32, fullscreen: bool) -> VkWindow {
    
    println!("Using Vulkan");
    
    let instance = {
      // Window specific extensions grabbed from vulkano_win
      let extensions = required_extensions();
      Instance::new(None, &extensions, None).expect("failed to create Vulkan instance")
    };
    
    let events_loop = winit::EventsLoop::new();
    let surface = {
      let temp_surface: Arc<Surface<winit::Window>>;
       
      if fullscreen {
       let monitor = {
         for (num, monitor) in events_loop.get_available_monitors().enumerate() {
           println!("Monitor #{}: {:?}", num, monitor.get_name());
         }

          let monitor = events_loop.get_available_monitors().nth(0).expect("Please enter a valid ID");

          println!("Using {:?}", monitor.get_name());

          monitor
        };
        
        // Fullscreen
        temp_surface = winit::WindowBuilder::new().with_fullscreen(Some(monitor))
                                           .with_title("Vulkan Fullscreen")
                                           .build_vk_surface(&events_loop, instance.clone())
                                           .unwrap()
      } else {
        // Windowed
        temp_surface = winit::WindowBuilder::new().with_dimensions(width, height)
                                            .with_min_dimensions(min_width, min_height)
                                           .with_title("Vulkan Windowed")
                                           .build_vk_surface(&events_loop, instance.clone())
                                           .unwrap()
      }
      temp_surface
    };
    
    println!("Winit Vulkan Window created");
    
    let (physical, queue) = {
      let mut found_suitable_device = false;
      
      let mut physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");
      
      for device in PhysicalDevice::enumerate(&instance) {
        physical = PhysicalDevice::from_index(&instance, device.index()).unwrap();
        
        for family in physical.queue_families() {
          if family.supports_graphics() && surface.is_supported(family).unwrap_or(false) {
           found_suitable_device = true;
           break;
          }
        }
        
        if found_suitable_device {
          println!("  {}: {} (type: {:?})", device.index(), device.name(), device.ty());
          break;
        }
      }
      
      let queue = physical.queue_families().find(|&q| {
          q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
        }
      ).expect("couldn't find a graphical queue family");
      
      (physical, queue)
    };
    
    let (device, mut queues) = {
      let device_ext = DeviceExtensions {
        khr_swapchain: true,
        .. DeviceExtensions::none()
      };
      
      Device::new(physical, physical.supported_features(), &device_ext, [(queue, 0.5)].iter().cloned()).expect("failed to create device")
    };
  
    let queue = queues.next().unwrap();
    let (swapchain, images) = {
      let caps = surface
                 .capabilities(physical)
                 .expect("failure to get surface capabilities");
      
      let settings = Settings::load();
      let min_width = settings.get_minimum_resolution()[0];
      let min_height = settings.get_minimum_resolution()[1];
      
      let dimensions = caps.current_extent.unwrap_or([min_width, min_height]);
                   
      let format = format::B8G8R8A8Unorm;//caps.supported_formats[0].0;//B8G8R8A8Unorm;
      let alpha = caps.supported_composite_alpha.iter().next().unwrap();//Opaque;
      let min_image_count = caps.min_image_count;
      let supported_usage_flags = caps.supported_usage_flags;
      
      println!("\nSwapchain:");
      println!("  Dimensions: {:?}", dimensions);
      println!("  Format: {:?}", format);
      
      Swapchain::new(device.clone(), surface.clone(), min_image_count, format,
                     dimensions, 1, supported_usage_flags, &queue,
                     SurfaceTransform::Identity, alpha, PresentMode::Fifo, true, None
                    ).expect("failed to create swapchain")
    };
    
    VkWindow {
      surface: surface,
      events: events_loop,
      queue: queue,
      device: device,
      swapchain: swapchain,
      images: images,
    }
  }
  
  /// Sets the title of the window
  pub fn set_title(&mut self, title: String) {
    self.surface.window().set_title(&title);
  }
  
  // Returns a clone of device
  pub fn get_device(&self) -> Arc<Device> {
    self.device.clone()
  }
  
  // Returns a clone of the queue
  pub fn get_queue(&self) -> Arc<Queue> {
    self.queue.clone()
  }
  
  // Returns the queue as a reference
  pub fn get_queue_ref(&self) -> &Arc<Queue> {
    &self.queue
  }
  
  // Recrates the swapchain to keep it relevant to the surface dimensions
  pub fn recreate_swapchain(&self, dimensions: [u32; 2]) -> Result<(Arc<Swapchain<winit::Window>>, Vec<Arc<SwapchainImage<winit::Window>>>), SwapchainCreationError> {
    let caps = self.surface
    .capabilities(self.device.physical_device())
    .expect("failure to get surface capabilities");
   
    let settings = Settings::load();
    let min_width = settings.get_minimum_resolution()[0];
    let min_height = settings.get_minimum_resolution()[1];
   
    let dimensions = caps.current_extent.unwrap_or([min_width, min_height]);
    println!("Window Resized!");
    self.swapchain.recreate_with_dimension(dimensions)
  }
  
  // Replaces entire swap chain memory with parameter swapchain
  pub fn replace_swapchain(&mut self, new_swapchain: Arc<Swapchain<winit::Window>>) {
    mem::replace(&mut self.swapchain, new_swapchain);
  }
  
  // Returns a reference to the current swapchain image
  pub fn get_images(&self) -> &Vec<Arc<SwapchainImage<winit::Window>>> {
    &self.images
  }
  
  // Replaces the current swapchain image with parameter image with mem::replace
  pub fn replace_images(&mut self, new_images: Vec<Arc<SwapchainImage<winit::Window>>>) {
    mem::replace(&mut self.images, new_images);
  }
  
  // Returns a clone of the swapchain
  pub fn get_swapchain(&self) -> Arc<Swapchain<winit::Window>> {
    self.swapchain.clone()
  }
  
  // Returns the current swapchain format enum from vulkano::format::Format
  pub fn get_swapchain_format(&self) -> format::Format {
    self.swapchain.format()
  }
  
  /// Returns the dimensions of the window as u32
  pub fn get_dimensions(&self) -> [u32; 2] {
    let (width, height) = self.surface.window().get_inner_size().unwrap();
    [width as u32, height as u32]
  }
  
  /// Returns a reference to the events loop
  pub fn get_events(&mut self) -> &mut EventsLoop {
    &mut self.events
  }
  
  /// Returns the current dpi scale factor
  ///
  /// Needed to solve issues with Hidpi monitors
  pub fn get_dpi_scale(&self) -> f32 {
    self.surface.window().hidpi_factor()
  }
  
  /// Enables the cursor to be drawn whilst over the window
  pub fn show_cursor(&mut self) {
    self.surface.window().set_cursor(winit::MouseCursor::Default);
  }
  
  /// Disables the cursor from being drawn whilst over the window
  pub fn hide_cursor(&mut self) {
    self.surface.window().set_cursor(winit::MouseCursor::NoneCursor);
  }
}
