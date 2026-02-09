#version 330 core
in vec2 v_texCoord;
out vec4 FragColor;

uniform sampler2D u_text;
uniform vec3 u_textColor;

void main()
{    
    // The texture atlas is a single-channel (red) texture.
    // We sample it, and use the red channel as the alpha value.
    // The color is determined by the u_textColor uniform.
    vec4 sampled = vec4(1.0, 1.0, 1.0, texture(u_text, v_texCoord).r);
    FragColor = vec4(u_textColor, 1.0) * sampled;
}
