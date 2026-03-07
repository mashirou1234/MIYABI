#version 330 core
layout (location = 0) in vec3 a_position;
layout (location = 1) in vec2 a_texCoord;
layout (location = 2) in vec3 a_normal;
layout (location = 3) in mat4 a_modelMatrix;

uniform mat4 u_view;
uniform mat4 u_projection;

out vec2 v_texCoord;
out vec3 v_worldNormal;

void main()
{
    vec4 world_position = a_modelMatrix * vec4(a_position, 1.0);
    mat3 normal_matrix = transpose(inverse(mat3(a_modelMatrix)));

    gl_Position = u_projection * u_view * world_position;
    v_texCoord = a_texCoord;
    v_worldNormal = normalize(normal_matrix * a_normal);
}
