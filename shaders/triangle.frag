#version 460 core

in vec3 vertColor;

out vec4 Color;

void main()
{
    Color = vec4(vertColor, 1.f);
}