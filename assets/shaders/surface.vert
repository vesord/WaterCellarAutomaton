#version 460 core

layout (location = 0) in vec3 Position;

out vec4 passColor;

uniform mat4 mvp_transform;

float invLerp(in float min, in float max, in float v) {
    return (v - min) / (max - min);
}

vec4 remap(in float iMin, in float iMax, in vec4 oMin, in vec4 oMax, in float v) {
    float t = invLerp(iMin, iMax, v);
    return mix(oMin, oMax, t);
}

const vec4 GRASS = vec4(0., .8, 0., 1.);
const vec4 ROCK = vec4(0.3, 0.22, 0.2, 1.);
const vec4 SNOW = vec4(0.9, 0.9, 0.9, 1.);
const float ROCK_LEVEL = 0.2;

void main()
{
    gl_Position = mvp_transform * vec4(Position, 1.0);

    float height = Position.y;

//    float red = -sin(height * 1.57059 + 1.57059) + 1.;
//    float green = abs(1.6 * height - 0.8) + 0.2;
//    float blue = -log(-2. * height + 2.) / 3. + 0.1;
//
//    passColor = vec4(red, green, blue, 1.);

    if (height < ROCK_LEVEL) { // TODO: add to uniform
        passColor = remap(0., ROCK_LEVEL, GRASS, ROCK, height);
    }
    else {
        passColor = remap(ROCK_LEVEL, 1., ROCK, SNOW, height);
    }
}