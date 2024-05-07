#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in float cornerID;

uniform mat4 mvp;

uniform vec3 pos;
uniform float time;
uniform float blockID;

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

    float pi = 3.1415926535897932384626433832795;

    float scale = 0.5;

    mat4 rotationMatrix = getRotationMatrix(0.0, mod(time, 2.0 * pi), 0.0);
    vec4 rotatedPosition = rotationMatrix * vec4(aPos * scale, 1.0);



    vec2 baseUV = vec2(mod(blockID, 16.0f) * 0.03308823529411764705882352941176f, 1.0f - floor((blockID/16.0f) * 0.52941176470588235294117647058824f));

    // Selecting UV based on cornerID
    if (cornerID == 0.0) {
        TexCoord = baseUV;
    } else if (cornerID == 1.0) {
        TexCoord = vec2(baseUV.x + (1.0f/128.0f), baseUV.y);
    } else if (cornerID == 2.0) {
        TexCoord = vec2(baseUV.x + (1.0f/128.0f), baseUV.y - (1.0f/128.0f));
    } else if (cornerID == 3.0) {
        TexCoord = vec2(baseUV.x, baseUV.y - (1.0f/128.0f));
    }

    gl_Position = mvp * (rotatedPosition + vec4(pos + vec3(0.0, sin(time) / 3.0, 0.0), 0.0));
}