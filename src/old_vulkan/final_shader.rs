use vulkano::buffer::BufferUsage;
use vulkano::buffer::ImmutableBuffer;
use vulkano::buffer::cpu_pool::CpuBufferPool;

use vulkano::sampler;
use vulkano::sync::NowFuture;
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;

use vulkano::command_buffer::DynamicState;
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::CommandBufferExecFuture;
use vulkano::command_buffer::AutoCommandBufferBuilder;

use vulkano::image as vkimage;
use vulkano::image::SwapchainImage;

use vulkano::device::Queue;
use vulkano::device::Device;

use vulkano::framebuffer;
use vulkano::framebuffer::RenderPassAbstract;

use vulkano::format;
use vulkano::format::ClearValue;

use vulkano::pipeline;
use vulkano::pipeline::GraphicsPipelineAbstract;

use crate::graphics::Vertex2d;
use crate::vulkan::TextureShader;
use crate::math;

use winit;

use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Matrix4;

use std::sync::Arc;

mod vs_final {
  vulkano_shaders::shader! {
    ty: "vertex",
    path: "src/shaders/glsl/VkFinal.vert"
  }
}

mod fs_final {
  vulkano_shaders::shader! {
    ty: "fragment",
    path: "src/shaders/glsl/VkFinal.frag"
  }
}

pub struct FinalShader {
  renderpass: Arc<RenderPassAbstract + Send + Sync>,
  pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
  framebuffer: Option<Vec<Arc<framebuffer::FramebufferAbstract + Send + Sync + Send + Sync>>>,
  uniformbuffer: CpuBufferPool<vs_final::ty::Data>,
  
  vertex_buffer: Arc<ImmutableBuffer<[Vertex2d]>>,
  index_buffer: Arc<ImmutableBuffer<[u32]>>,
  
  descriptor_pool: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  
  sampler: Arc<sampler::Sampler>,
}

impl FinalShader {
  pub fn create(device: Arc<Device>, queue: Arc<Queue>, swapchain_format: format::Format) -> (FinalShader, Vec<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>) {
    let uniformbuffer = CpuBufferPool::<vs_final::ty::Data>::new(device.clone(), BufferUsage::uniform_buffer());
    
    let vs_final = vs_final::Shader::load(device.clone()).expect("failed to create shader module");
    let fs_final = fs_final::Shader::load(device.clone()).expect("failed to create shader module");
    
    let renderpass = Arc::new(single_pass_renderpass!(device.clone(),
      attachments: {
        out_colour: {
          load: DontCare,
          store: Store,
          format: swapchain_format,
          samples: 1,
        }
      },
      pass: {
        color: [out_colour],
        depth_stencil: {},
        resolve: [],
      }
    ).unwrap());
    
    let pipeline:  Arc<GraphicsPipelineAbstract + Send + Sync> = Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
        .vertex_shader(vs_final.main_entry_point(), ())
        //.viewports_dynamic_scissors_irrelevant(1)
        .viewports_scissors_dynamic(1)
        .fragment_shader(fs_final.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(renderpass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());
    
    let (vertex_buffer, future_vtx) = TextureShader::create_vertex(Arc::clone(&queue));
    let (idx_buffer, future_idx) = TextureShader::create_index(queue);
    
    let sampler = sampler::Sampler::new(device.clone(), sampler::Filter::Linear,
                                                   sampler::Filter::Linear, 
                                                   sampler::MipmapMode::Nearest,
                                                   sampler::SamplerAddressMode::ClampToEdge,
                                                   sampler::SamplerAddressMode::ClampToEdge,
                                                   sampler::SamplerAddressMode::ClampToEdge,
                                                   0.0, 1.0, 0.0, 0.0).unwrap();
    
    let descriptor_set = FixedSizeDescriptorSetsPool::new(Arc::clone(&pipeline), 0);
    
    (
      FinalShader {
        renderpass: renderpass,
        pipeline: pipeline,
        framebuffer: None,
        uniformbuffer: uniformbuffer,
        
        descriptor_pool: descriptor_set,
        
        vertex_buffer: vertex_buffer,
        index_buffer: idx_buffer,
        sampler: sampler,
      },
      vec!(future_idx, future_vtx)
    )
  }
  
  pub fn empty_framebuffer(&mut self) {
    self.framebuffer = None;
  }
  
  pub fn recreate_framebuffer(&mut self, images: &Vec<Arc<SwapchainImage<winit::Window>>>) {
    if self.framebuffer.is_none() {
      let new_framebuffer = Some(images.iter().map( |image| {
             let fb = framebuffer::Framebuffer::start(self.renderpass.clone())
                      .add(image.clone()).unwrap()
                      .build().unwrap();
             Arc::new(fb) as Arc<framebuffer::FramebufferAbstract + Send + Sync>
             }).collect::<Vec<_>>());
      self.framebuffer = new_framebuffer;
    }
  }
  
  pub fn begin_renderpass(&mut self, cb: AutoCommandBufferBuilder, secondary: bool, image_num: usize) -> AutoCommandBufferBuilder {
    cb.begin_render_pass(self.framebuffer.as_ref().unwrap()[image_num].clone(), secondary, vec![ClearValue::None]).unwrap()
  }
  
  pub fn draw(&mut self, cb: AutoCommandBufferBuilder, dynamic_state: &DynamicState, dimensions: [f32; 2], texture_projection: Matrix4<f32>, texture_image: Arc<vkimage::AttachmentImage>) -> AutoCommandBufferBuilder {
    
    let model = math::calculate_texture_model(Vector3::new(dimensions[0] as f32*0.5, dimensions[1] as f32*0.5, 0.0), Vector2::new(dimensions[0] as f32, dimensions[1] as f32), 90.0);
    
    let uniform_data = vs_final::ty::Data {
      projection: texture_projection.into(),
      model: model.into(),
    };
    
    let uniform_subbuffer;
   // if self.uniformbuffer.capacity() > 2 {
  //    if let Some(buffer) = self.uniformbuffer.try_next(uniform_data) {
  //      uniform_subbuffer = buffer;
  //    } else {
  //      return cb;
  //    }
   // } else {
      uniform_subbuffer = self.uniformbuffer.next(uniform_data).unwrap();
  //  }
    
    let vertex = Arc::clone(&self.vertex_buffer);
    let index = Arc::clone(&self.index_buffer);
    let pipeline = Arc::clone(&self.pipeline);
    
    let descriptor_set = self.descriptor_pool.next()
                             .add_buffer(uniform_subbuffer.clone()).unwrap()
                             .add_sampled_image(Arc::clone(&texture_image),
                                               Arc::clone(&self.sampler)).unwrap()
                             .build().unwrap();
    
    let acb = cb.draw_indexed(pipeline, dynamic_state, vec!(vertex), index, descriptor_set, ()).unwrap();
    acb
   //cb
  }
  
  pub fn end_renderpass(&mut self, cb: AutoCommandBufferBuilder) -> AutoCommandBufferBuilder {
    cb.end_render_pass().unwrap()
  }
}