#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 uv;

uniform mat4 mvp;
out vec2 TexCoord;
void main() {
    TexCoord = uv;
    gl_Position = mvp * vec4(aPos, 1.0);
}