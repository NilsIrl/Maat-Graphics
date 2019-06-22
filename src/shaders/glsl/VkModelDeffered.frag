#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 v_camera_pos;
layout(location = 2) in vec4 v_camera_center;
layout(location = 3) in vec4 v_camera_up;
layout(location = 4) in vec3 v_light_positions[3];
layout(location = 7) in vec3 v_light_colours[3];
layout(location = 10) in float v_light_intensity[3];
layout(location = 13) in vec3 v_sun_direction;
layout(location = 14) in vec4 v_sun_colour;

layout(location = 0) out vec4 outColour;

layout (input_attachment_index = 1, binding = 1) uniform subpassInput colour_texture;
layout (input_attachment_index = 2, binding = 2) uniform subpassInput mro_texture;
layout (input_attachment_index = 3, binding = 3) uniform subpassInput emissive_texture;
layout (input_attachment_index = 4, binding = 4) uniform subpassInput normal_texture;
layout (input_attachment_index = 5, binding = 5) uniform subpassInput depth_texture;
layout (input_attachment_index = 6, binding = 6) uniform subpassInput position_texture;

const float M_PI = 3.141592653589793;

float cot(float value) {
  return 1.0 / tan(value);
}

float to_radians(float degree) {
  return degree * (M_PI/180.0);
}

float D_GGX(float dotNH, float roughness) {
  float alpha = roughness * roughness;
  float alpha2 = alpha * alpha;
  float denom = dotNH * dotNH * (alpha2 - 1.0) + 1.0;
  return (alpha2)/(M_PI * denom*denom); 
}

float G_SchlicksmithGGX(float dotNL, float dotNV, float roughness) {
  float r = (roughness + 1.0);
  float k = (r*r) / 8.0;
  float GL = dotNL / (dotNL * (1.0 - k) + k);
  float GV = dotNV / (dotNV * (1.0 - k) + k);
  
  return GL * GV;
}

vec3 F_Schlick(float cosTheta, float metallic) {
  vec3 F0 = mix(vec3(0.04), vec3(0.2, 0.2, 0.2), metallic);
  vec3 F = F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0); 
  
  return F; 
}

vec3 BRDF(vec3 L, vec3 V, vec3 N, float metallic, float roughness, vec3 light_position, vec3 light_colour, float intensity, vec3 world_pos) {
  vec3 H = normalize(V+L);
  float dotNV = clamp(dot(N, V), 0.0, 1.0);
  float dotNL = clamp(dot(N, L), 0.0, 1.0);
  float dotLH = clamp(dot(L, H), 0.0, 1.0);
  float dotNH = clamp(dot(N, H), 0.0, 1.0);
  
  vec3 colour = vec3(0.0);
  
  float distance = length(v_light_positions[0] - world_pos);//length(light_position-world_pos);
  
  float attenuation = 1.0;
  attenuation *= 1.0 / max(distance * distance, 0.01*0.01);
  
  vec3 radiance = light_colour * intensity;// * attenuation;
  
  if (dotNL > 0.0) {
    float rr = max(0.05, roughness);
    
    float D = D_GGX(dotNH, roughness);
    
    float G = G_SchlicksmithGGX(dotNL, dotNV, roughness);
    
    vec3 F = F_Schlick(dotNV, metallic);
    
    vec3 spec = D *F * G / (4.0 * dotNL * dotNV);
    
    colour += spec * radiance * dotNL; // * light_colour;
  }
  
  return colour;
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

vec3 depth_to_world_position(float depth_value, vec3 camera_ray, mat4 invProjection, mat4 invView) {
  /*float x = uvs.x;
  float y = -uvs.y;
  
  vec4 clip_coords = vec4(x,y,depth_value, 1.0);
  
  vec4 eye_matrix = invProjection * clip_coords;
  vec4 eye_coords = vec4(eye_matrix.xyz, 0.0);
  
  vec4 world_matrix = invView * eye_coords;
  
  vec3 world_pos = world_matrix.xyz;
  
  return world_pos;*/
  
  vec4 position = subpassLoad(position_texture);
  vec3 cam_facing = normalize(camera_ray);
  vec3 depth_dir = cam_facing * (depth_value * 2.0 - 1.0);
  
  vec4 clip_space = vec4(vec3(uvs.x* 2.0 - 1.0, -1.0*uvs.y* 2.0 - 1.0, 0.0) + depth_dir, 1.0);
  
  vec4 unproject = clip_space * invProjection;
  vec4 unview = unproject * invView;
  
  vec3 world_pos = unproject.xyz / unproject.w;
  
  return world_pos;
}


void main() {
  vec4 base_colour = subpassLoad(colour_texture);
  
  if (base_colour.a == 0.0) {
    discard;
  }
  
  float aspect = v_camera_center.w/v_camera_up.w;
  mat4 projection = create_perspective_matrix(v_camera_pos.w, aspect, 0.1, 256.0);
  mat4 view = create_view_matrix(v_camera_pos.xyz, v_camera_center.xyz, v_camera_up.xyz);
  mat4 invProjection = inverse(projection);
  mat4 invView = inverse(view);
  
  float depth =  subpassLoad(depth_texture).x;
  
  vec3 camera_ray = v_camera_pos.xyz - v_camera_center.xyz;
  
  vec3 world_pos = depth_to_world_position(depth, camera_ray, invProjection, invView);
  //vec3 world_pos = subpassLoad(position_texture).rgb * 100.0 - 100.0;
  vec3 N = vec3(subpassLoad(normal_texture).rgb) * 2.0 - 1.0;
  vec3 V = normalize(v_camera_pos.xyz - world_pos);
  
  vec3 Lo = vec3(0.0);
  
  vec3 L[3];
  L[0] = normalize(v_light_positions[0] - world_pos);
  L[1] = normalize(v_light_positions[1] - world_pos);
  L[2] = normalize(v_light_positions[2] - world_pos);
  
  
  vec4 mro_colour = subpassLoad(mro_texture);
  
  for(int i = 0; i < 3; ++i) {
    Lo += BRDF(L[i], V, N, mro_colour.x, mro_colour.y, v_light_positions[i], v_light_colours[i], v_light_intensity[i], world_pos);
  }
  
  base_colour.rgb *= 0.02;
  base_colour.rgb += Lo;
  
  base_colour.rgb = pow(base_colour.rgb, vec3(0.4545));
  
  outColour = base_colour;
}

/*
void main() {
  float aspect = v_camera_center.w/v_camera_up.w;
  mat4 projection = create_perspective_matrix(v_camera_pos.w, aspect, 0.1, 256.0);
  mat4 view = create_view_matrix(v_camera_pos.xyz, v_camera_center.xyz, v_camera_up.xyz);
  
  vec3 frag_pos = subpassLoad(position_texture).rgb;
  vec3 N = normalize(vec3((subpassLoad(normal_texture).rgb - vec3(0.5)) * 2.0));
  
  mat4 identity = mat4(1.0);
  
  vec3 light_position = vec3(view * vec4(v_light_positions[0], 1.0));
  vec3 camera_position = vec3(view *  vec4(v_camera_pos.xyz, 1.0));
  
  vec3 V = normalize(camera_position - frag_pos);
  
  vec3 Lo = vec3(0.0);
  
  vec3 L = normalize(light_position - frag_pos);
  //vec3 L[3];
  //L[0] = normalize(v_light_positions[0] - world_pos);
  //L[1] = normalize(v_light_positions[1] - world_pos);
  //L[2] = normalize(v_light_positions[2] - world_pos);
  
  vec4 base_colour = subpassLoad(colour_texture);
  vec4 mro_colour = subpassLoad(mro_texture);
  
   Lo += BRDF(L, V, N, mro_colour.x, mro_colour.y, light_position, v_light_colours[0], v_light_intensity[0], frag_pos);
  
  //for(int i = 0; i < 3; ++i) {
   // Lo += BRDF(L[i], V, N, mro_colour.x, mro_colour.y, v_light_positions[i], v_light_colours[i], v_light_intensity[i], world_pos);
 // }
  
  base_colour *= 0.02;
  base_colour.rgb += Lo;
  
  base_colour.rgb = pow(base_colour.rgb, vec3(0.4545));
  
  outColour = vec4(base_colour.rgb, 1.0);
}*/
