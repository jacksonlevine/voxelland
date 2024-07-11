#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 uv;

uniform mat4 mvp;

uniform vec3 pos;
uniform vec3 lastpos;

uniform float scale;



uniform float xrot;
uniform float yrot;
uniform float zrot;

uniform vec3 lastrot;


uniform float interp_time;

uniform float walkbob;

out vec2 TexCoord;

mat4 getRotationMatrix(float xrot, float yrot, float zrot) {
    mat4 Rx = mat4(1.0, 0.0, 0.0, 0.0,
                   0.0, cos(xrot), -sin(xrot), 0.0,
                   0.0, sin(xrot), cos(xrot), 0.0,
                   0.0, 0.0, 0.0, 1.0);
                   
    mat4 Ry = mat4(cos(yrot), 0.0, sin(yrot), 0.0,
                   0.0, 1.0, 0.0, 0.0,
                   -sin(yrot), 0.0, cos(yrot), 0.0,
                   0.0, 0.0, 0.0, 1.0);
                   
    mat4 Rz = mat4(cos(zrot), -sin(zrot), 0.0, 0.0,
                   sin(zrot), cos(zrot), 0.0, 0.0,
                   0.0, 0.0, 1.0, 0.0,
                   0.0, 0.0, 0.0, 1.0);
    
    return Rz * Ry * Rx; // Note: The order might need to be adjusted based on your specific needs
}


void main() {

    vec3 mixedrots = mix(lastrot, vec3(xrot, yrot, zrot), min(interp_time*4.0, 1.0));

    mat4 rotationMatrix = getRotationMatrix(mixedrots.x, mixedrots.y, mixedrots.z);
    vec4 rotatedPosition = rotationMatrix * vec4(aPos * scale, 1.0);

    TexCoord = uv;

    vec3 bob = vec3(0.0, ((sin(walkbob) )/20.0), 0.0) + vec3(0.0, 0.5, 0.0);
    
    gl_Position = mvp * (rotatedPosition + vec4(mix(lastpos, pos, min(interp_time*4.0, 1.0)), 0.0) + vec4(bob * -1.0, 0.0));
}