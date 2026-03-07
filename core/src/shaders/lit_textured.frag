#version 330 core
out vec4 FragColor;

in vec2 v_texCoord;
in vec3 v_worldNormal;

uniform sampler2D u_texture;
uniform vec3 u_lightDirection;
uniform vec3 u_lightColor;
uniform float u_ambientStrength;
uniform float u_diffuseStrength;

void main()
{
    vec4 albedo = texture(u_texture, v_texCoord);
    float lambert = max(dot(normalize(v_worldNormal), normalize(-u_lightDirection)), 0.0);
    vec3 lighting = vec3(u_ambientStrength) + (u_lightColor * lambert * u_diffuseStrength);
    FragColor = vec4(albedo.rgb * clamp(lighting, 0.0, 1.0), albedo.a);
}
