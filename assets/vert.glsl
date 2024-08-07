#version 450 core
layout (location = 0) in uint u32;
layout (location = 1) in uint eightbit;
layout (location = 2) in uint rgb;
out vec3 vertexColor;
out vec2 TexCoord;
out vec3 pos;

uniform vec2 chunkpos;
uniform mat4 mvp;
uniform vec3 camPos;
uniform float ambientBrightMult;
uniform float viewDistance;
uniform float planet_y;

uniform float walkbob;

void main()
{

    // Decode the RGB value from the 16-bit attribute
    uint r = (rgb & 0xF00) >> 8;
    uint g = (rgb & 0x0F0) >> 4;
    uint b = (rgb & 0x00F);

    vec3 color = vec3(float(r) / 15.0, float(g) / 15.0, float(b) / 15.0);

    float lx = float((u32 >> 28) & 0x0000000F);       // Top 4 bits for x
    float ly = float((u32 >> 20) & 0x000000FF);      // Next 8 bits for y
    float lz = float((u32 >> 16) & 0x0000000F);       // Next 4 bits for z

    vec3 position = vec3(lx, ly, lz) + (vec3(chunkpos.x, 0, chunkpos.y) * 15) + vec3(0.0, planet_y, 0.0);

    uint cornerID = ((u32 >> 12) & 0x0000000F);  // Next 4 bits for corner
    float ambientBright = float((u32 >> 8) & 0x0000000F); // Next 4 bits for al
    float blockBright = float((u32 >> 4) & 0x0000000F);   // Next 4 bits for bl

    blockBright = blockBright / 1.25;

    //Texture stuff
    float onePixel = 0.00183823529411764705882352941176f;     //  1/544      Padding
    float textureWidth = 0.02941176470588235294117647058824f; // 16/544      16 pixel texture width
    float texSlotWidth = 0.03308823529411764705882352941176f;

    vec2 texOffsets[6] = {
        vec2(onePixel, -onePixel),
        vec2(onePixel + textureWidth, -onePixel),
        vec2(onePixel + textureWidth, -(onePixel + textureWidth)),
        vec2(onePixel + textureWidth, -(onePixel + textureWidth)),
        vec2(onePixel, -(onePixel + textureWidth)),
        vec2(onePixel, -onePixel)
    };


    // Unpack from the eightbit and cast to float
    float u = float((eightbit >> 4) & 0xF);  // Top 4 bits for u
    float v = float(eightbit & 0xF);         // Lower 4 bits for v

    vec2 uvOffset = texOffsets[cornerID];
    vec2 uv = vec2((u * texSlotWidth) + uvOffset.x, (1.0f - (v * texSlotWidth)) + uvOffset.y);
    

    float ambBright = ambientBrightMult * ambientBright;

    float distance = pow(distance(position, camPos)/(5), 2)/5.0f;
    vec3 bob = vec3(0.0, ((sin(walkbob) )/20.0), 0.0) + vec3(0.0, 0.3, 0.0);
    gl_Position = mvp * (vec4(position - bob , 1.0) );

    float bright = min(16.0f,ambBright);

    

    vertexColor = vec3(min((bright/16.0f) + color.r, 1.0), min((bright/16.0f) + color.g, 1.0), min((bright/16.0f) + color.b, 1.0) );
    //vertexColor = vec3(lx / 10.0, ly / 10.0, 1.0);  // Assuming maximum values for normalization
    TexCoord = uv;
    pos = position;

}