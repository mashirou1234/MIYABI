#version 330 core
layout (location = 0) in vec3 a_position;
layout (location = 1) in vec2 a_texCoord;

// A mat4 is 4 vec4s, so it takes up 4 attribute locations.
// We start at location 2 since 0 and 1 are taken.
layout (location = 2) in mat4 a_modelMatrix;

uniform mat4 u_view;
uniform mat4 u_projection;

out vec2 v_texCoord;

void main()
{
    gl_Position = u_projection * u_view * a_modelMatrix * vec4(a_position, 1.0);
    v_texCoord = a_texCoord;
}
