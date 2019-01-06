#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 v_uvs;
layout(location = 1) out vec4 v_new_colour;
layout(location = 2) out vec3 v_outlineColour;
layout(location = 3) out vec4 v_edge_width;

layout(set = 0, binding = 1) uniform Data {
  mat4 scale;
  mat4 projection;
} uniforms;

layout(push_constant) uniform PushConstants {
  mat4 model;
  vec4 letter_uv;
  vec4 edge_width; 
  vec4 colour;
  vec4 outline_colour;
} push_constants;

void main() {
  vec2 new_uv = uv;
  vec2 new_pos = position;
  
  if(uv.x == 0) {
    new_uv.x += push_constants.letter_uv.x;
    new_pos.x = 0;
  } else
  if(uv.x == 1) {
    new_uv.x = push_constants.letter_uv.z;
    new_pos.x = push_constants.letter_uv.z - push_constants.letter_uv.x;
  }
  
  if(uv.y == 0) {
    new_uv.y = push_constants.letter_uv.w;
    new_pos.y = 0;
  } else
  if(uv.y == 1) {
    new_uv.y = push_constants.letter_uv.y;
    new_pos.y = push_constants.letter_uv.w - push_constants.letter_uv.y;
  }
  
  gl_Position = uniforms.projection * uniforms.scale * push_constants.model * vec4(new_pos, 0.0, 1.0);
  
  v_uvs = new_uv;
  v_outlineColour = push_constants.outline_colour.rgb;
  v_new_colour = push_constants.colour;
  v_edge_width = push_constants.edge_width;
}