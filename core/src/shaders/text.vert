#version 330 core
layout (location = 0) in vec4 a_vertex; // vec2 position, vec2 texCoords
out vec2 v_texCoord;

uniform mat4 u_projection;

void main()
{
    gl_Position = u_projection * vec4(a_vertex.xy, 0.0, 1.0);
    v_texCoord = a_vertex.zw;
}
