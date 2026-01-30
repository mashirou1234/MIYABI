#version 330 core
layout (location = 0) in vec3 aPos;

uniform vec3 u_translation;

void main()
{
   gl_Position = vec4(aPos + u_translation, 1.0);
}
