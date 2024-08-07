#version 330 core

precision highp float;

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 uv;

uniform mat4 mvp;

uniform float scale;

uniform vec3 camPos;

out vec2 TexCoord;

uniform float walkbob;


void main() {


    TexCoord = uv;
    gl_Position = mvp * vec4((aPos * vec3(8.0, 1.0, 8.0)) + vec3(camPos.x, 24.6, camPos.z), 1.0);
}