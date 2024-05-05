#version 330 core
out vec4 FragColor;
in vec2 TexCoord;
in float elementID;
uniform sampler2D ourTexture;
uniform float mousedOverElement;
uniform float clickedOnElement;

void main() {
    FragColor = texture(ourTexture, TexCoord);
    if(FragColor.a < 0.1) {
        discard;
    }

    if(clickedOnElement == elementID) {
        FragColor = vec4(vec3(1.0, 1.0, 1.0) - FragColor.rgb, 1.0);
    } else if(mousedOverElement == elementID) {
        FragColor = FragColor + vec4(0.3, 0.3, 0.3, 0.0);
    }
}