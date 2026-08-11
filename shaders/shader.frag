/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

#version 450

#pragma shader_stage(fragment)

layout(location = 0) out vec4 outColor;

layout(location = 0) in vec2 fragTexCoord;

layout(binding = 2) uniform sampler2D texSampler;

void main() {
    outColor = texture(texSampler, fragTexCoord);
}