#version 330 core
out vec4 FragColor;
in vec2 TexCoord;
in float elementID;
uniform sampler2D ourTexture;

uniform float mousedSlot;

void main() {
    FragColor = texture(ourTexture, TexCoord);
    if(FragColor.a < 0.1) {
        discard;
    }

    if(mousedSlot == elementID) {
        FragColor = FragColor + vec4(0.3, 0.3, 0.3, 0.0);
    }

}