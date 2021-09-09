#version 460 core

layout (location = 0) in vec3 Position;

out vec4 passColor;

uniform mat4 mvp_transform;

void main()
{
    gl_Position = mvp_transform * vec4(Position, 1.0);
    passColor = vec4(abs(gl_Position.z), 0., 0., 1.);
}