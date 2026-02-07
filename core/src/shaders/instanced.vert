#version 330 core
layout (location = 0) in vec3 a_position;

// A mat4 is 4 vec4s, so it takes up 4 attribute locations (1, 2, 3, 4).
layout (location = 1) in mat4 a_modelMatrix;

uniform mat4 u_view;
uniform mat4 u_projection;

void main()
{
    // Apply all transformations
    gl_Position = u_projection * u_view * a_modelMatrix * vec4(a_position, 1.0);
}
