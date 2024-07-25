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

    const vec2 TEXS[45] = vec2[45](
    vec2(0.0, 0.0),  // 0
    vec2(1.0, 0.0),  // 1 sand
    vec2(2.0, 0.0),  // 2 water
    vec2(3.0, 0.0),  // 3 grass
    vec2(4.0, 0.0),  // 4 dirt
    vec2(5.0, 0.0),  // 5 cobble
    vec2(6.0, 0.0),  // 6 log
    vec2(7.0, 0.0),  // 7 leaves
    vec2(8.0, 0.0),  // 8 glass
    vec2(9.0, 0.0),  // 9 smooth stone
    vec2(10.0, 0.0), // 10 planks wood
    vec2(7.0, 1.0),  // 11 bush leaves
    vec2(4.0, 2.0),  // 12 petrified wood
    vec2(6.0, 2.0),  // 13 red stone
    vec2(7.0, 2.0),  // 14 salted earth
    vec2(8.0, 2.0),  // 15 bedrock
    vec2(0.0, 3.0),  // 16 red crystal unattainable
    vec2(0.0, 4.0),  // 17 red crystal
    vec2(12.0, 1.0), // 18 light
    vec2(12.0, 0.0), // 19 door
    vec2(0.0, 1.0),  // 20 ladder
    vec2(15.0, 0.0), // 21 chest
    vec2(13.0, 1.0), // 22 bamboo
    vec2(1.0, 3.0),  // 23 tallgrass
    vec2(10.0, 2.0), // 24 blue light
    vec2(11.0, 2.0), // 25 purple light
    vec2(12.0, 2.0), // 26 yellow light
    vec2(13.0, 2.0), // 27 red light
    vec2(10.0, 3.0), // 28 green light
    vec2(11.0, 3.0), // 29 orange light
    vec2(12.0, 3.0), // 30 teal light
    vec2(1.0, 5.0),  // 31 crafttable
    vec2(3.0, 3.0),  // 32 apple
    vec2(2.0, 3.0),  // 33 bamboo chute
    vec2(7.0, 4.0),  // 34 dead leaves
    vec2(2.0, 4.0),  // 35 metal rock
    vec2(2.0, 5.0),  // 36 crude blade
    vec2(3.0, 5.0),  // 37 crude pick
    vec2(4.0, 5.0),  // 38 crude mattock
    vec2(5.0, 5.0),   // 39 crude axe

    vec2(10.0, 4.0),  // 40 jumper blue
    vec2(11.0, 4.0),  // 41 jumper yellow
    vec2(10.0, 5.0),   // 42 trampoline block

    vec2(0.0, 8.0),  // 43 rubbertree wood
    vec2(1.0, 8.0)   // 44 rubbertree leaves
);



    float pi = 3.1415926535897932384626433832795;

    float scale = 0.5;

    mat4 rotationMatrix = getRotationMatrix(0.0, mod(time, 2.0 * pi), 0.0);
    vec4 rotatedPosition = rotationMatrix * vec4(aPos * scale, 1.0);

    vec2 buv = TEXS[int(blockID)];

    vec2 baseUV = vec2(mod(buv.x, 16.0f) * 0.03308823529411764705882352941176f, 1.0f - ((buv.y/16.0f) * 0.52941176470588235294117647058824f));

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