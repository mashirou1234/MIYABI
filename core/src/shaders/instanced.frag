#version 330 core
out vec4 FragColor;

void main()
{
    // For now, all objects will be rendered in solid white.
    // In the future, this could come from a material uniform.
    FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}
