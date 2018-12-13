use vk;

use crate::modules::Device;
use crate::modules::RenderPass;
use crate::modules::Pipeline;
use crate::modules::DescriptorSet;
use crate::modules::CommandPool;
use crate::ownage::check_errors;

use std::mem;
use std::ptr;

pub struct CommandBuffer {
  command_buffer: vk::CommandBuffer,
}

impl CommandBuffer {
  pub fn primary(device: &Device, command_pool: &CommandPool) -> CommandBuffer {
    let command_pool = command_pool.local_command_pool();
    
    let command_buffer_allocate_info = {
      vk::CommandBufferAllocateInfo {
        sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
        pNext: ptr::null(),
        commandPool: *command_pool,
        level: vk::COMMAND_BUFFER_LEVEL_PRIMARY,
        commandBufferCount: 1,
      }
    };
    
    let mut command_buffer: vk::CommandBuffer = unsafe { mem::uninitialized() };
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      check_errors(vk.AllocateCommandBuffers(*device, &command_buffer_allocate_info, &mut command_buffer));
    }
    
    CommandBuffer {
      command_buffer,
    }
  }
  
  pub fn from_buffer(command_buffer: vk::CommandBuffer) -> CommandBuffer {
    CommandBuffer {
      command_buffer,
    }
  }
  
  pub fn secondary(device: &Device, command_pool: &CommandPool) -> CommandBuffer {
    let command_pool = command_pool.local_command_pool();
    
    let command_buffer_allocate_info = {
      vk::CommandBufferAllocateInfo {
        sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
        pNext: ptr::null(),
        commandPool: *command_pool,
        level: vk::COMMAND_BUFFER_LEVEL_SECONDARY,
        commandBufferCount: 1,
      }
    };
    
    let mut command_buffer: vk::CommandBuffer = unsafe { mem::uninitialized() };
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      check_errors(vk.AllocateCommandBuffers(*device, &command_buffer_allocate_info, &mut command_buffer));
    }
    
    CommandBuffer {
      command_buffer,
    }
  }
  
  pub fn begin_render_pass(&self, device: &Device, render_pass: &RenderPass, framebuffer: &vk::Framebuffer, clear_values: &Vec<vk::ClearValue>, width: u32, height: u32) {
    let vk = device.pointers();
    
    let mut render_pass_begin_info = {
      vk::RenderPassBeginInfo {
        sType: vk::STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
        pNext: ptr::null(),
        renderPass: *render_pass.internal_object(),
        framebuffer: *framebuffer,
        renderArea: vk::Rect2D { offset: vk::Offset2D {x: 0, y: 0 }, extent: vk::Extent2D { width: width, height: height, } },
        clearValueCount: clear_values.len() as u32,
        pClearValues: clear_values.as_ptr(),
      }
    };
    
    unsafe {
      vk.CmdBeginRenderPass(self.command_buffer, &render_pass_begin_info, vk::SUBPASS_CONTENTS_INLINE);
    }
  }
  
  pub fn end_render_pass(&self, device: &Device) {
    let vk = device.pointers();
    
    unsafe {
      vk.CmdEndRenderPass(self.command_buffer);
    }
  }
  
  pub fn begin_command_buffer(&self, device: &Device, flags: u32) {
    let vk = device.pointers();
    
    let command_buffer_begin_info = {
      vk::CommandBufferBeginInfo {
        sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
        pNext: ptr::null(),
        flags: flags,
        pInheritanceInfo: ptr::null(),
      }
    };
    
    unsafe {
      check_errors(vk.BeginCommandBuffer(self.command_buffer, &command_buffer_begin_info));
    }
  }
  
  pub fn end_command_buffer(&self, device: &Device) {
    let vk = device.pointers();
    
    unsafe {
      check_errors(vk.EndCommandBuffer(self.command_buffer));
    }
  }
  
  pub fn bind_descriptor_set(&self, device: &Device, pipeline: &Pipeline, descriptor_set: &DescriptorSet) {
    let vk = device.pointers();
    
    unsafe {
      vk.CmdBindDescriptorSets(self.command_buffer, vk::PIPELINE_BIND_POINT_GRAPHICS, *pipeline.layout(), 0, 1, descriptor_set.set(), 0, ptr::null());
    }
  }
  
  pub fn bind_pipeline(&self, device: &Device, pipeline: &Pipeline) {
    let vk = device.pointers();
    
    unsafe {
      vk.CmdBindPipeline(self.command_buffer, vk::PIPELINE_BIND_POINT_GRAPHICS, *pipeline.pipeline(0));
    }
  }
  
  pub fn bind_vertex_buffer(&self, device: &Device, vertex_buffer: &vk::Buffer) {
    let vk = device.pointers();
    
    unsafe {
      vk.CmdBindVertexBuffers(self.command_buffer, 0, 1, vertex_buffer, &0);
    }
  }
  
  pub fn bind_index_buffer(&self, device: &Device, index_buffer: &vk::Buffer) {
    let vk = device.pointers();
    
    unsafe {
      vk.CmdBindIndexBuffer(self.command_buffer, *index_buffer, 0, vk::INDEX_TYPE_UINT32);
    }
  }
  
  pub fn draw_indexed(&self, device: &Device, index_count: u32, instance_count: u32) {
    let vk = device.pointers();
    
    unsafe {
      vk.CmdDrawIndexed(self.command_buffer, index_count, instance_count, 0, 0, 0);
    }
  }
  
  pub fn internal_object(&self) -> &vk::CommandBuffer {
    &self.command_buffer
  }
  
  pub fn destroy(&self) {
    
  }
}
