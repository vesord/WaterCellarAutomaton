#version 410 core

layout (location = 0) in vec3 Position;

out vec4 vertColor;

uniform mat4 mvp_transform;

void main()
{
    gl_Position = mvp_transform * vec4(Position, 1.0);
    vertColor = vec4(0., 0., 1., 0.9);
}