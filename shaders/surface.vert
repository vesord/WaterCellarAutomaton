#version 460 core

layout (location = 0) in vec3 Position;

out vec4 passColor;

void main()
{
    gl_Position = vec4(Position, 1.0);
    passColor = gl_Position;
}