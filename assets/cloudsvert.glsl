#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 uv;

uniform mat4 mvp;

uniform float scale;

uniform vec3 camPos;

out vec2 TexCoord;



void main() {


    TexCoord = uv;
    gl_Position = mvp * vec4((aPos * vec3(5.0, 1.0, 5.0)) + vec3(camPos.x, 0.0, camPos.z), 1.0);
}