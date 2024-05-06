#version 450 core
in vec2 TexCoord;
out vec4 FragColor;
uniform sampler2D ourTexture;
void main()
{
    vec4 texColor = texture(ourTexture, TexCoord);
    FragColor = texColor;
    if(FragColor.a == 0.0f) {
        discard;
    }
}