#version 330 core
layout (location = 0) in vec3 position;
uniform mat4 mvp;
uniform vec3 translation;

void main()
{

    gl_Position = mvp * vec4((position + translation + vec3(0.5, 0.5, 0.5)), 1.0);

}