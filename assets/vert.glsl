#version 450 core
layout (location = 0) in uint u32;
layout (location = 1) in uint eightbit;
out vec3 vertexColor;
out vec2 TexCoord;
out vec3 pos;
uniform vec2 chunkpos;
uniform mat4 mvp;
uniform vec3 camPos;
uniform float ambientBrightMult;
uniform float viewDistance;
void main()
{

    

    float lx = float((u32 >> 28) & 0x0000000F);       // Top 4 bits for x
    float ly = float((u32 >> 20) & 0x000000FF);      // Next 8 bits for y
    float lz = float((u32 >> 16) & 0x0000000F);       // Next 4 bits for z

    vec3 position = vec3(lx, ly, lz) + (vec3(chunkpos.x, 0, chunkpos.y) * 15);

    uint cornerID = ((u32 >> 12) & 0x0000000F);  // Next 4 bits for corner
    float ambientBright = float((u32 >> 8) & 0x0000000F); // Next 4 bits for al
    float blockBright = float((u32 >> 4) & 0x0000000F);   // Next 4 bits for bl

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
    gl_Position = mvp * (vec4(position , 1.0) - vec4(0.5, 0.5, 0.5, 0.0));

    float bright = min(16.0f, blockBright + ambBright);

    

    vertexColor = vec3(bright/16.0f, bright/16.0f, bright/16.0f);
    //vertexColor = vec3(lx / 10.0, ly / 10.0, 1.0);  // Assuming maximum values for normalization
    TexCoord = uv;
    pos = position;
}