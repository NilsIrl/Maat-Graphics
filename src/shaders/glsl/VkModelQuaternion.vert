#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;
layout(location = 3) in vec4 colour;
layout(location = 4) in vec4 tangent;

layout(location = 0) out vec2 uvs;
layout(location = 1) out vec4 v_colour;
layout(location = 2) out vec4 v_base_colour_factor;
layout(location = 3) out vec4 v_alpha_cutoff; // alpha, cutoff, _, use emissive
layout(location = 4) out vec3 v_normal;
layout(location = 5) out vec3 v_world_pos;
layout(location = 6) out vec3 v_camera_pos;
layout(location = 10) out vec3 v_scanline;
layout(location = 11) out vec4 v_use_textures;
layout(location = 12) out vec2 v_mr;

layout(set = 0, binding = 0) uniform UniformBuffer {
  vec4 use_textures; //base, metallic_roughness, normal, occlusion
  vec4 emissive_alpha; //use_emissive, normal_scale, alpha_cutoff, alpha_mask
  vec4 base_colour_factor; // r, g, b, a
  vec4 mro_factors; // metallic_factor, roughness_factor, occlusion_string, _
  vec4 emissive_factor; // r, g, b, _
} uniforms;

layout(push_constant) uniform PushConstants {
  vec4 c_position; // x, y, z, fov
  vec4 c_center;   // x, y, z, aspect
  vec4 c_up;       // x, y, z, x_scale
  vec4 model;      // x, y, z, y_scale
  vec4 quaternion;   // qx, qy, qz, qw 
  vec4 hologram_scanline; // hologram_enabled, scanline, z_scale, _
} push_constants;

const float M_PI = 3.141592653589793;

float cot(float value) {
  return 1.0 / tan(value);
}

float to_radians(float degree) {
  return degree * (M_PI/180.0);
}

vec3 to_radians(vec3 degrees) {
  return vec3(to_radians(degrees.x), to_radians(degrees.y), to_radians(degrees.z));
}

mat4 create_perspective_matrix(float fov, float aspect, float near, float far) {
  float f = cot(to_radians(fov) / 2.0);
  
  mat4 perspective = mat4(
                      vec4(f / aspect, 0.0,   0.0,                               0.0),
                      vec4(0.0,        f,     0.0,                               0.0),
                      vec4(0.0,        0.0,   (far + near) / (near - far),      -1.0),
                      vec4(0.0,        0.0,   (2.0 * far * near) / (near - far), 0.0)
                    );
                
  return perspective;
}

// center is a point not a direction
mat4 create_view_matrix(vec3 eye, vec3 center, vec3 up) {
  vec3 dir = center - eye;
  
  vec3 f = normalize(dir);
  vec3 s = normalize(cross(f, up));
  vec3 u = cross(s,f);
  
  mat4 look_at_matrix = mat4(vec4(s.x,           u.x,        -f.x,         0.0), 
                             vec4(s.y,           u.y,        -f.y,         0.0), 
                             vec4(s.z,           u.z,        -f.z,         0.0), 
                             vec4(-dot(eye, s), -dot(eye, u), dot(eye, f), 1.0));
  
  return look_at_matrix;
}


/*
mat4 create_rotation_matrix(vec3 deg_rotation) {
  vec3 rotation = to_radians(deg_rotation);
  
  float s = sin(rotation.x);
  float c = cos(rotation.x);

  mat4 rot_x = mat4(vec4(1.0,  0.0, 0.0, 0.0), 
                    vec4(0.0,  c,   s,   0.0), 
                    vec4(0.0, -s,   c,   0.0), 
                    vec4(0.0,  0.0, 0.0, 1.0));
  
  s = sin(rotation.y);
  c = cos(rotation.y);
  
  mat4 rot_y = mat4(vec4(c,   0.0, -s,   0.0), 
                    vec4(0.0, 1.0,  0.0, 0.0), 
                    vec4(s,   0.0,  c,   0.0), 
                    vec4(0.0, 0.0,  0.0, 1.0));
  
  s = sin(rotation.z);
  c = cos(rotation.z);
  
  mat4 rot_z = mat4(vec4(c,   s,   0.0, 0.0), 
                    vec4(-s,  c,   0.0, 0.0), 
                    vec4(0.0, 0.0, 1.0, 0.0), 
                    vec4(0.0, 0.0, 0.0, 1.0));
  
  mat4 rotation_matrix = rot_y*rot_x*rot_z;
  
  return rotation_matrix;
}*/

mat4 create_translation_matrix(vec3 pos) {
  mat4 translate_matrix = mat4(vec4(1.0, 0.0, 0.0, 0.0), 
                               vec4(0.0, 1.0, 0.0, 0.0), 
                               vec4(0.0, 0.0, 1.0, 0.0), 
                               vec4(pos,           1.0));
  
  return translate_matrix;
}

mat4 create_scale_matrix(vec3 scale) {
  mat4 scale_matrix = mat4(vec4(scale.x, 0.0,     0.0,     0.0), 
                               vec4(0.0,     scale.y, 0.0,     0.0), 
                               vec4(0.0,     0.0,     scale.z, 0.0), 
                               vec4(0.0,     0.0,     0.0,     1.0));
  
  return scale_matrix;
}

void main() {
  vec3 model_scale = vec3(push_constants.c_up.w, push_constants.model.w, push_constants.rotation.w);
  
  mat4 projection = create_perspective_matrix(push_constants.c_position.w, push_constants.c_center.w, 0.1, 1080.0);
  mat4 view = create_view_matrix(push_constants.c_position.xyz, push_constants.c_center.xyz, push_constants.c_up.xyz);
  mat4 model = create_translation_matrix(push_constants.model.xyz);
  mat4 scale = create_scale_matrix(model_scale);
  mat4 rotation = create_rotation_matrix(push_constants.rotation.xyz);
  
  vec3 local_pos = vec3(model * rotation * scale * vec4(position, 1.0));
  
  vec4 rotated_normal = rotation * vec4(-normal.x, normal.y, normal.z, 1.0);
  
  uvs = uv;
  v_colour = colour;
  v_alpha_cutoff = vec4(uniforms.emissive_alpha.z, uniforms.emissive_alpha.w, 0.0, uniforms.emissive_alpha.x);
  v_base_colour_factor = uniforms.base_colour_factor;
  v_world_pos = local_pos;
  v_normal = rotated_normal.xyz;
  
  v_camera_pos = push_constants.c_position.rgb;
  
  v_use_textures = uniforms.use_textures;
  v_scanline = vec3(push_constants.hologram_scanline.y, local_pos.y, push_constants.hologram_scanline.x);
  v_mr = vec2(uniforms.mro_factors.x, uniforms.mro_factors.y);
  
  gl_Position = projection * view * vec4(local_pos, 1.0);
}
